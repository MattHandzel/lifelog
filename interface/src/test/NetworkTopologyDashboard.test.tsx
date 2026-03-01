import { beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import NetworkTopologyDashboard from '../components/NetworkTopologyDashboard';
import { clearInvokeMocks, mockInvoke } from './setup';

describe('NetworkTopologyDashboard', () => {
  beforeEach(() => {
    clearInvokeMocks();
    localStorage.clear();
  });

  it('renders collector nodes and sends component config updates', async () => {
    const setConfigSpy = vi.fn(async () => undefined);

    mockInvoke('get_system_state', () => [
      {
        collector_id: 'collector-alpha',
        name: 'collector-alpha',
        last_seen_secs: Math.floor(Date.now() / 1000),
        total_buffer_size: 32,
        upload_lag_bytes: 1024,
        last_upload_time_secs: Math.floor(Date.now() / 1000) - 15,
        source_states: ['screen:active', 'audio:capture'],
        source_buffer_sizes: [],
      },
    ]);

    mockInvoke('get_component_config', (args) => {
      const input = args as { componentType: string };
      if (input.componentType === 'screen') return { enabled: true };
      if (input.componentType === 'microphone') return { enabled: true };
      return { enabled: false };
    });

    mockInvoke('set_component_config', setConfigSpy);

    render(<NetworkTopologyDashboard />);

    await waitFor(() => {
      expect(screen.getByTestId('collector-node-collector-alpha')).toBeInTheDocument();
    });

    const disableButtons = screen.getAllByRole('button', { name: 'Disable' });
    fireEvent.click(disableButtons[0]);

    await waitFor(() => {
      expect(setConfigSpy).toHaveBeenCalled();
    });

    const calls = setConfigSpy.mock.calls as unknown as Array<[unknown]>;
    const firstCall = calls[0];
    expect(firstCall).toBeDefined();
    const payload = firstCall[0] as {
      collectorId: string;
      componentType: string;
      configValue: { enabled: boolean };
    };

    expect(payload.collectorId).toBe('collector-alpha');
    expect(payload.componentType).toBe('screen');
    expect(payload.configValue.enabled).toBe(false);
  });
});
