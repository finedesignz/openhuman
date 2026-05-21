//! Claude Code CLI provider.
//!
//! Drives Anthropic's `claude` CLI (`--print --output-format stream-json
//! --verbose --resume <uuid>`) instead of calling the HTTP API directly.
//! Tools are exposed back into the CLI over MCP so OpenHuman's native
//! Rust tools remain authoritative.
//!
//! v1 surface (this PR scaffold): version probe, auth resolution, shared
//! types. `chat()` returns a clear NotImplemented error until Phase 2
//! lands the driver + stream parser.

pub mod auth;
pub mod types;
pub mod version_check;

use async_trait::async_trait;

use super::traits::{ChatMessage, Provider, ProviderCapabilities};

/// Provider string prefix used in the factory grammar: `claude-code:<model>`.
pub const PROVIDER_PREFIX: &str = "claude-code:";

/// Scaffold provider — refuses chat requests with a clear error so callers
/// can surface "CC driver not yet implemented" while we land Phase 2.
pub struct ClaudeCodeProvider {
    pub model: String,
}

impl ClaudeCodeProvider {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
        }
    }
}

#[async_trait]
impl Provider for ClaudeCodeProvider {
    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            native_tool_calling: true,
            vision: false,
            ..ProviderCapabilities::default()
        }
    }

    async fn chat_with_system(
        &self,
        _system_prompt: Option<&str>,
        _message: &str,
        _model: &str,
        _temperature: f64,
    ) -> anyhow::Result<String> {
        anyhow::bail!(
            "[claude-code] driver not yet implemented (Phase 2). \
             Provider scaffold loaded for model={}",
            self.model
        )
    }

    async fn chat_with_history(
        &self,
        _messages: &[ChatMessage],
        _model: &str,
        _temperature: f64,
    ) -> anyhow::Result<String> {
        anyhow::bail!(
            "[claude-code] chat_with_history not yet implemented (Phase 2). model={}",
            self.model
        )
    }
}
