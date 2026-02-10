import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import ReplayDashboard from '../components/ReplayDashboard';
import { mockInvoke, clearInvokeMocks } from './setup';

describe('ReplayDashboard', () => {
  beforeEach(() => {
    clearInvokeMocks();
  });

  afterEach(() => {
    clearInvokeMocks();
  });

  it('renders replay header and controls', () => {
    render(<ReplayDashboard />);
    expect(screen.getByText('Replay')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /load replay/i })).toBeInTheDocument();
    expect(screen.getByPlaceholderText(/leave blank to auto-pick/i)).toBeInTheDocument();
    expect(screen.getByPlaceholderText(/Browser,Ocr/i)).toBeInTheDocument();
  });

  it('calls replay and renders steps', async () => {
    let capturedArgs: unknown = null;

    mockInvoke('replay', (args) => {
      capturedArgs = args;
      return [
        {
          start: 1736935200,
          end: 1736935260,
          screen_key: { uuid: '00000000-0000-0000-0000-000000000001', origin: 'laptop:Screen' },
          context_keys: [{ uuid: '00000000-0000-0000-0000-000000000002', origin: 'laptop:Browser' }],
        },
      ];
    });

    mockInvoke('get_screenshots_data', () => [
      {
        uuid: '00000000-0000-0000-0000-000000000001',
        timestamp: 1736935200,
        width: 100,
        height: 100,
        dataUrl: 'data:image/png;base64,AAAA',
        mime_type: 'image/png',
      },
    ]);

    render(<ReplayDashboard />);

    const startLabel = screen.getByText('Start');
    const startInput = startLabel.parentElement!.querySelector('input[type="datetime-local"]')!;
    fireEvent.change(startInput, { target: { value: '2025-01-15T10:00' } });

    const endLabel = screen.getByText('End');
    const endInput = endLabel.parentElement!.querySelector('input[type="datetime-local"]')!;
    fireEvent.change(endInput, { target: { value: '2025-01-15T10:05' } });

    fireEvent.click(screen.getByRole('button', { name: /load replay/i }));

    await waitFor(() => {
      expect(screen.getByText(/1 steps/i)).toBeInTheDocument();
    });

    await waitFor(() => {
      expect(capturedArgs).toBeDefined();
      const args = capturedArgs as Record<string, unknown>;
      expect(args.startTime).toBeTypeOf('number');
      expect(args.endTime).toBeTypeOf('number');
    });

    // Should render context count for the step ("1 ctx").
    expect(screen.getByText(/1 ctx/i)).toBeInTheDocument();
  });
});

