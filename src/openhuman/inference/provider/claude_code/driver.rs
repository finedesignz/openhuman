//! Spawn the `claude` CLI for one chat turn, stream its stdout into the
//! event mapper, and return an aggregated `ChatResponse`.
//!
//! The driver does *not* own concurrency limits; the `ClaudeCodeProvider`
//! holds a `Semaphore` and acquires a permit before calling this. The
//! driver also does *not* own MCP — Phase 3 will wire `--mcp-config`.

use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::sync::mpsc;

use super::event_mapper::EventMapper;
use super::input_builder::build_stdin;
use super::session_store::{generate_uuid_v4, is_uuid_v4, SessionStore};
use super::stream_parser::StreamJsonParser;
use crate::openhuman::inference::provider::traits::{
    ChatMessage, ChatResponse, ProviderDelta,
};

/// One CC chat turn.
pub struct TurnContext<'a> {
    pub bin_path: PathBuf,
    pub workspace_dir: PathBuf,
    pub thread_id: String,
    pub model: String,
    pub append_system_prompt: Option<String>,
    pub messages: &'a [ChatMessage],
    pub session_store: Arc<SessionStore>,
    pub stream: Option<&'a mpsc::Sender<ProviderDelta>>,
    /// Optional explicit `ANTHROPIC_API_KEY` to set on the child. When
    /// `None`, the CLI falls back to its own `~/.claude/.credentials.json`.
    pub anthropic_api_key: Option<String>,
}

/// Run one turn against the `claude` CLI. Awaits process exit. Forwards
/// `ProviderDelta`s through `ctx.stream` as they arrive and returns the
/// aggregated `ChatResponse` when done.
pub async fn run_turn(ctx: TurnContext<'_>) -> anyhow::Result<ChatResponse> {
    let stored = ctx.session_store.get(&ctx.thread_id);
    let is_new = !stored.as_deref().map(is_uuid_v4).unwrap_or(false);
    let cc_session_id = if is_new {
        let id = generate_uuid_v4();
        if let Err(e) = ctx.session_store.set(&ctx.thread_id, &id) {
            log::warn!(
                "[claude-code][driver] failed to persist session uuid for thread {}: {}",
                ctx.thread_id,
                e
            );
        }
        id
    } else {
        stored.expect("checked Some above")
    };

    let mut args: Vec<String> = vec![
        "-p".into(),
        "--input-format".into(),
        "stream-json".into(),
        "--output-format".into(),
        "stream-json".into(),
        "--verbose".into(),
        "--include-partial-messages".into(),
        "--add-dir".into(),
        ctx.workspace_dir.display().to_string(),
        if is_new {
            "--session-id".into()
        } else {
            "--resume".into()
        },
        cc_session_id.clone(),
        "--model".into(),
        ctx.model.clone(),
    ];
    if let Some(sp) = ctx.append_system_prompt.as_ref().filter(|s| !s.trim().is_empty()) {
        args.push("--append-system-prompt".into());
        args.push(sp.clone());
    }

    log::debug!(
        "[claude-code][driver] spawn bin={} model={} is_new={} cc_session_id={}",
        ctx.bin_path.display(),
        ctx.model,
        is_new,
        cc_session_id
    );

    let mut cmd = Command::new(&ctx.bin_path);
    cmd.args(&args)
        .current_dir(&ctx.workspace_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(key) = &ctx.anthropic_api_key {
        cmd.env("ANTHROPIC_API_KEY", key);
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| anyhow::anyhow!("failed to spawn `claude`: {e}"))?;

    // Write stdin
    let stdin_bytes = build_stdin(ctx.messages, is_new);
    if stdin_bytes.is_empty() {
        anyhow::bail!("[claude-code][driver] no input messages to deliver");
    }
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(&stdin_bytes)
            .await
            .map_err(|e| anyhow::anyhow!("write stdin: {e}"))?;
        stdin
            .shutdown()
            .await
            .map_err(|e| anyhow::anyhow!("close stdin: {e}"))?;
    }

    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("claude child stdout missing"))?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow::anyhow!("claude child stderr missing"))?;

    let mut parser = StreamJsonParser::new();
    let mut mapper = EventMapper::new();
    let mut buf = [0u8; 8192];

    // Drain stderr in parallel into a buffer for diagnostics.
    let stderr_task = tokio::spawn(async move {
        let mut acc = String::new();
        let mut tmp = [0u8; 4096];
        while let Ok(n) = stderr.read(&mut tmp).await {
            if n == 0 {
                break;
            }
            acc.push_str(&String::from_utf8_lossy(&tmp[..n]));
            if acc.len() > 16_384 {
                acc.truncate(16_384);
            }
        }
        acc
    });

    loop {
        let n = stdout
            .read(&mut buf)
            .await
            .map_err(|e| anyhow::anyhow!("read stdout: {e}"))?;
        if n == 0 {
            break;
        }
        for ev in parser.feed_bytes(&buf[..n]) {
            for delta in mapper.handle(ev) {
                if let Some(tx) = ctx.stream {
                    let _ = tx.send(delta).await;
                }
            }
        }
    }
    for ev in parser.end() {
        for delta in mapper.handle(ev) {
            if let Some(tx) = ctx.stream {
                let _ = tx.send(delta).await;
            }
        }
    }

    let status = child
        .wait()
        .await
        .map_err(|e| anyhow::anyhow!("wait child: {e}"))?;
    let stderr_text = stderr_task.await.unwrap_or_default();

    if !status.success() {
        anyhow::bail!(
            "[claude-code][driver] exit {:?} stderr={}",
            status.code(),
            stderr_text.trim()
        );
    }
    if let Some(err) = mapper.error.clone() {
        anyhow::bail!("[claude-code][driver] {}", err);
    }

    Ok(mapper.into_response())
}
