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
    expect(screen.getByRole('button', { name: /previous step/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /play replay/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /next step/i })).toBeInTheDocument();
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

    expect(screen.getByText(/1 ctx/i)).toBeInTheDocument();
    expect(screen.getByText(/1\/1 frames buffered/i)).toBeInTheDocument();
  });

  it('shows filtered keystrokes overlay for active window', async () => {
    mockInvoke('replay', () => [
      {
        start: 1736935200,
        end: 1736935260,
        screen_key: { uuid: 'screen-1', origin: 'laptop:Screen' },
        context_keys: [
          { uuid: 'window-1', origin: 'laptop:WindowActivity' },
          { uuid: 'keys-match', origin: 'laptop:Keystrokes' },
          { uuid: 'keys-other', origin: 'laptop:Keystrokes' },
          { uuid: 'clip-1', origin: 'laptop:Clipboard' },
        ],
      },
    ]);

    mockInvoke('get_screenshots_data', () => [
      {
        uuid: 'screen-1',
        timestamp: 1736935200,
        width: 100,
        height: 100,
        dataUrl: 'data:image/png;base64,AAAA',
        mime_type: 'image/png',
      },
    ]);

    mockInvoke('get_frame_data', () => [
      {
        uuid: 'window-1',
        modality: 'WindowActivity',
        timestamp: 1736935201,
        text: null,
        url: null,
        title: null,
        visit_count: null,
        command: null,
        working_dir: null,
        exit_code: null,
        application: 'Code',
        window_title: 'Replay.tsx',
        duration_secs: 1,
        audio_data_url: null,
        codec: null,
        sample_rate: null,
        channels: null,
        audio_duration_secs: null,
        image_data_url: null,
        width: null,
        height: null,
        mime_type: null,
        camera_device: null,
        processes: null,
      },
      {
        uuid: 'keys-match',
        modality: 'Keystrokes',
        timestamp: 1736935202,
        text: 'typed in replay window',
        url: null,
        title: null,
        visit_count: null,
        command: null,
        working_dir: null,
        exit_code: null,
        application: 'Code',
        window_title: 'Replay.tsx',
        duration_secs: null,
        audio_data_url: null,
        codec: null,
        sample_rate: null,
        channels: null,
        audio_duration_secs: null,
        image_data_url: null,
        width: null,
        height: null,
        mime_type: null,
        camera_device: null,
        processes: null,
      },
      {
        uuid: 'keys-other',
        modality: 'Keystrokes',
        timestamp: 1736935203,
        text: 'typed elsewhere',
        url: null,
        title: null,
        visit_count: null,
        command: null,
        working_dir: null,
        exit_code: null,
        application: 'Browser',
        window_title: 'Other',
        duration_secs: null,
        audio_data_url: null,
        codec: null,
        sample_rate: null,
        channels: null,
        audio_duration_secs: null,
        image_data_url: null,
        width: null,
        height: null,
        mime_type: null,
        camera_device: null,
        processes: null,
      },
      {
        uuid: 'clip-1',
        modality: 'Clipboard',
        timestamp: 1736935204,
        text: 'copied text',
        url: null,
        title: null,
        visit_count: null,
        command: null,
        working_dir: null,
        exit_code: null,
        application: null,
        window_title: null,
        duration_secs: null,
        audio_data_url: null,
        codec: null,
        sample_rate: null,
        channels: null,
        audio_duration_secs: null,
        image_data_url: null,
        width: null,
        height: null,
        mime_type: null,
        camera_device: null,
        processes: null,
      },
    ]);

    render(<ReplayDashboard />);

    const startInput = screen.getByText('Start').parentElement!.querySelector('input[type="datetime-local"]')!;
    const endInput = screen.getByText('End').parentElement!.querySelector('input[type="datetime-local"]')!;

    fireEvent.change(startInput, { target: { value: '2025-01-15T10:00' } });
    fireEvent.change(endInput, { target: { value: '2025-01-15T10:05' } });

    fireEvent.click(screen.getByRole('button', { name: /load replay/i }));

    await waitFor(() => {
      expect(screen.getByText('typed in replay window')).toBeInTheDocument();
    });

    expect(screen.queryByText('typed elsewhere')).not.toBeInTheDocument();
    expect(screen.getByText('copied text')).toBeInTheDocument();
  });
});
