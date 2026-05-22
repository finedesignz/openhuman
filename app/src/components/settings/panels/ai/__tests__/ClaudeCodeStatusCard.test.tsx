import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { ClaudeCodeStatusCard } from '../ClaudeCodeStatusCard';

const probe = vi.fn();

vi.mock('../../../../../utils/tauriCommands/config', () => ({
  openhumanClaudeCodeStatus: () => probe(),
}));

describe('ClaudeCodeStatusCard', () => {
  beforeEach(() => {
    probe.mockReset();
  });

  it('renders the installed version + path when CC is OK', async () => {
    probe.mockResolvedValueOnce({
      result: { status: 'ok', version: '2.0.4', path: '/usr/local/bin/claude' },
    });
    render(<ClaudeCodeStatusCard />);
    await waitFor(() => {
      expect(screen.getByText(/Installed \(2\.0\.4\)/)).toBeInTheDocument();
    });
    expect(screen.getByText('/usr/local/bin/claude')).toBeInTheDocument();
  });

  it('shows the install hint when the binary is missing', async () => {
    probe.mockResolvedValueOnce({ result: { status: 'not_installed' } });
    render(<ClaudeCodeStatusCard />);
    await waitFor(() => {
      expect(screen.getByText(/Claude Code CLI is not installed/i)).toBeInTheDocument();
    });
  });

  it('shows the outdated state with min_required', async () => {
    probe.mockResolvedValueOnce({
      result: {
        status: 'outdated',
        version: '1.9.0',
        min_required: '2.0.0',
        path: '/usr/local/bin/claude',
      },
    });
    render(<ClaudeCodeStatusCard />);
    await waitFor(() => {
      expect(screen.getByText(/Outdated — found 1\.9\.0, need ≥ 2\.0\.0/)).toBeInTheDocument();
    });
  });

  it('surfaces a probe error', async () => {
    probe.mockRejectedValueOnce(new Error('boom'));
    render(<ClaudeCodeStatusCard />);
    await waitFor(() => {
      expect(screen.getByText(/Failed to probe: boom/)).toBeInTheDocument();
    });
  });

  it('re-probes when Refresh is clicked', async () => {
    probe
      .mockResolvedValueOnce({ result: { status: 'not_installed' } })
      .mockResolvedValueOnce({ result: { status: 'ok', version: '2.0.4', path: '/x/y/claude' } });
    const user = userEvent.setup();
    render(<ClaudeCodeStatusCard />);
    await waitFor(() => {
      expect(screen.getByText(/Claude Code CLI is not installed/i)).toBeInTheDocument();
    });
    await user.click(screen.getByRole('button', { name: /Refresh/i }));
    await waitFor(() => {
      expect(screen.getByText(/Installed \(2\.0\.4\)/)).toBeInTheDocument();
    });
    expect(probe).toHaveBeenCalledTimes(2);
  });
});
