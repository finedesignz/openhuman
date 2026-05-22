import { useCallback, useEffect, useState } from 'react';

import {
  type ClaudeCodeStatus,
  openhumanClaudeCodeStatus,
} from '../../../../utils/tauriCommands/config';

/**
 * Status card for the Claude Code CLI provider.
 *
 * Probes the local `claude` binary on mount (and on a manual Refresh) and
 * surfaces install / version state to the user. Read-only — does not write
 * any settings. Embed inside the AI settings panel above the routing
 * dropdowns once per-role selection wiring lands.
 */
export function ClaudeCodeStatusCard() {
  const [status, setStatus] = useState<ClaudeCodeStatus | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState<boolean>(false);

  const probe = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const resp = await openhumanClaudeCodeStatus();
      setStatus(resp.result);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setStatus(null);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void probe();
  }, [probe]);

  return (
    <section
      data-testid="claude-code-status-card"
      className="rounded-lg border border-neutral-200 bg-white p-4 dark:border-neutral-800 dark:bg-neutral-900">
      <header className="mb-2 flex items-center justify-between">
        <h3 className="text-sm font-semibold text-neutral-900 dark:text-neutral-100">
          Claude Code CLI
        </h3>
        <button
          type="button"
          onClick={() => {
            void probe();
          }}
          disabled={loading}
          className="text-xs text-neutral-500 hover:text-neutral-900 disabled:opacity-50 dark:text-neutral-400 dark:hover:text-neutral-100">
          {loading ? 'Probing…' : 'Refresh'}
        </button>
      </header>
      <StatusBody status={status} error={error} />
      <p className="mt-3 text-xs text-neutral-500 dark:text-neutral-400">
        Use the <code>claude-code:&lt;model&gt;</code> provider string to route chat, agentic, or
        reasoning workloads through your local Claude Code CLI install.
      </p>
    </section>
  );
}

function StatusBody({ status, error }: { status: ClaudeCodeStatus | null; error: string | null }) {
  if (error) {
    return <p className="text-xs text-rose-600 dark:text-rose-400">Failed to probe: {error}</p>;
  }
  if (!status) {
    return <p className="text-xs text-neutral-500 dark:text-neutral-400">Probing…</p>;
  }
  switch (status.status) {
    case 'ok':
      return (
        <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-xs">
          <dt className="text-neutral-500">Status</dt>
          <dd className="text-emerald-600 dark:text-emerald-400">Installed ({status.version})</dd>
          <dt className="text-neutral-500">Path</dt>
          <dd className="font-mono text-neutral-700 dark:text-neutral-300">{status.path}</dd>
        </dl>
      );
    case 'not_installed':
      return (
        <p className="text-xs text-amber-600 dark:text-amber-400">
          Claude Code CLI is not installed. Install via{' '}
          <code>npm install -g @anthropic-ai/claude-code</code> or follow{' '}
          <a
            href="https://docs.anthropic.com/en/docs/claude-code"
            target="_blank"
            rel="noreferrer noopener"
            className="underline hover:text-amber-700 dark:hover:text-amber-300">
            Anthropic's docs
          </a>
          .
        </p>
      );
    case 'outdated':
      return (
        <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-xs">
          <dt className="text-neutral-500">Status</dt>
          <dd className="text-rose-600 dark:text-rose-400">
            Outdated — found {status.version}, need ≥ {status.min_required}
          </dd>
          <dt className="text-neutral-500">Path</dt>
          <dd className="font-mono text-neutral-700 dark:text-neutral-300">{status.path}</dd>
        </dl>
      );
    case 'unusable':
      return (
        <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-xs">
          <dt className="text-neutral-500">Status</dt>
          <dd className="text-rose-600 dark:text-rose-400">Unusable — {status.reason}</dd>
          <dt className="text-neutral-500">Path</dt>
          <dd className="font-mono text-neutral-700 dark:text-neutral-300">{status.path}</dd>
        </dl>
      );
  }
}
