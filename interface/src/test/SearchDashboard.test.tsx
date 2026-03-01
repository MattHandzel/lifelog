import { beforeEach, describe, expect, it } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import SearchDashboard from '../components/SearchDashboard';
import { clearInvokeMocks, mockInvoke } from './setup';

function buildFrame(overrides: Record<string, unknown>) {
  return {
    uuid: 'default-uuid',
    modality: 'Ocr',
    timestamp: 1700000000,
    text: null,
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
    ...overrides,
  };
}

describe('SearchDashboard', () => {
  beforeEach(() => {
    clearInvokeMocks();
  });

  it('loads frame previews and highlights matching snippet terms', async () => {
    let frameRequestArgs: unknown = null;
    mockInvoke('query_timeline', () => [
      { uuid: 'ocr-1', origin: 'collector:ocr', modality: 'ocr', timestamp: null },
    ]);
    mockInvoke('get_frame_data_thumbnails', (args) => {
      frameRequestArgs = args;
      return [
        buildFrame({
          uuid: 'ocr-1',
          modality: 'Ocr',
          text: 'The quick brown fox jumps over the lazy dog and keeps running.',
        }),
      ];
    });

    const { container } = render(<SearchDashboard />);
    fireEvent.change(screen.getByPlaceholderText('Search for files, images, audio...'), {
      target: { value: 'brown' },
    });
    fireEvent.click(screen.getByRole('button', { name: /^search$/i }));

    await waitFor(() => {
      expect(screen.getByText(/quick/i)).toBeInTheDocument();
    });

    const args = frameRequestArgs as { keys: Array<{ uuid: string; origin: string }> };
    expect(args.keys).toEqual([{ uuid: 'ocr-1', origin: 'collector:ocr' }]);

    const highlight = container.querySelector('mark');
    expect(highlight).not.toBeNull();
    expect(highlight).toHaveTextContent(/brown/i);
  });

  it('renders thumbnail images for screen results', async () => {
    mockInvoke('query_timeline', () => [
      { uuid: 'screen-1', origin: 'collector:screen', modality: 'screen', timestamp: null },
    ]);
    mockInvoke('get_frame_data_thumbnails', () => [
      buildFrame({
        uuid: 'screen-1',
        modality: 'Screen',
        image_data_url: 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAAB',
      }),
    ]);

    const { container } = render(<SearchDashboard />);
    fireEvent.change(screen.getByPlaceholderText('Search for files, images, audio...'), {
      target: { value: 'screen' },
    });
    fireEvent.click(screen.getByRole('button', { name: /^search$/i }));

    await waitFor(() => {
      const img = container.querySelector('img[loading="lazy"]');
      expect(img).not.toBeNull();
      expect(img).toHaveAttribute('loading', 'lazy');
      expect(img).toHaveAttribute('src', expect.stringContaining('data:image/'));
    });
  });
});
