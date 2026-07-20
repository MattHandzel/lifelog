import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { renderHook, waitFor, act } from '@testing-library/react';
import { useModalityFrames } from '../lib/useModalityFrames';
import { mockInvoke, clearInvokeMocks } from './setup';

// A timeline the backend returns in an arbitrary (NOT newest-first) order, so that
// the sort option is actually exercised rather than coinciding with backend order.
const TIMELINE = [
  { uuid: 'a', origin: 'Camera', modality: 'Camera', timestamp: 100 },
  { uuid: 'c', origin: 'Camera', modality: 'Camera', timestamp: 300 },
  { uuid: 'b', origin: 'Camera', modality: 'Camera', timestamp: 200 },
  { uuid: 'x', origin: 'Screen', modality: 'Screen', timestamp: 999 },
  { uuid: 'd', origin: 'Camera', modality: 'Camera', timestamp: 400 },
];

function mockTimeline(): void {
  mockInvoke('query_timeline', () => TIMELINE);
  // Echo one frame per requested key so `frames` order mirrors `entries` order.
  mockInvoke('get_frame_data', (args) => {
    const keys = (args as { keys: Array<{ uuid: string; origin: string }> }).keys;
    return keys.map((k) => ({ uuid: k.uuid, timestamp: null, image_data_url: null }));
  });
}

describe('useModalityFrames', () => {
  beforeEach(() => clearInvokeMocks());
  afterEach(() => clearInvokeMocks());

  it('preserves backend order by default (sort: none) — the Processes contract', async () => {
    mockTimeline();
    const { result } = renderHook(() => useModalityFrames('Camera'));
    await waitFor(() => expect(result.current.loading).toBe(false));
    // Backend Camera order as returned: a(100), c(300), b(200), d(400) — untouched.
    expect(result.current.entries.map((e) => e.uuid)).toEqual(['a', 'c', 'b', 'd']);
  });

  it('limit (no sort) slices the first N of backend order — newest-first contract', async () => {
    mockTimeline();
    const { result } = renderHook(() => useModalityFrames('Camera', { limit: 2 }));
    await waitFor(() => expect(result.current.loading).toBe(false));
    // Backend already returns newest-first; limit takes the first N without reordering.
    expect(result.current.entries.map((e) => e.uuid)).toEqual(['a', 'c']);
  });

  it('sort: timestamp-desc reorders matched entries newest-first', async () => {
    mockTimeline();
    const { result } = renderHook(() =>
      useModalityFrames('Camera', { sort: 'timestamp-desc' }),
    );
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.entries.map((e) => e.uuid)).toEqual(['d', 'c', 'b', 'a']);
  });

  it('sort + limit caps after ordering (Audio: newest 50 shape, scaled down)', async () => {
    mockTimeline();
    const { result } = renderHook(() =>
      useModalityFrames('Camera', { sort: 'timestamp-desc', limit: 2 }),
    );
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.entries.map((e) => e.uuid)).toEqual(['d', 'c']);
  });

  it('offset + limit pages through ordered entries (Camera pagination shape)', async () => {
    mockTimeline();
    // pageSize 2, page 2 → offset 2, limit 2 → 3rd and 4th newest.
    const { result } = renderHook(() =>
      useModalityFrames('Camera', { sort: 'timestamp-desc', offset: 2, limit: 2 }),
    );
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.entries.map((e) => e.uuid)).toEqual(['b', 'a']);
  });

  it('dedups a reload fired while one is already in flight', async () => {
    let timelineCalls = 0;
    let releaseTimeline: (v: unknown) => void = () => {};
    mockInvoke('query_timeline', () => {
      timelineCalls += 1;
      return new Promise((resolve) => {
        releaseTimeline = resolve;
      });
    });
    mockInvoke('get_frame_data', () => []);

    const { result } = renderHook(() => useModalityFrames('Camera', { autoLoad: false }));

    // First reload hangs on the pending query_timeline; the second must be a no-op.
    await act(async () => {
      void result.current.reload();
      void result.current.reload();
    });
    expect(timelineCalls).toBe(1);

    // Let the in-flight load finish so nothing dangles.
    await act(async () => {
      releaseTimeline([]);
    });
  });

  it('caches per-uuid — a refetch of the same entries skips get_frame_data', async () => {
    let frameDataCalls = 0;
    mockInvoke('query_timeline', () => TIMELINE);
    mockInvoke('get_frame_data', (args) => {
      frameDataCalls += 1;
      const keys = (args as { keys: Array<{ uuid: string; origin: string }> }).keys;
      return keys.map((k) => ({ uuid: k.uuid, timestamp: null, image_data_url: null }));
    });

    const { result } = renderHook(() =>
      useModalityFrames('Camera', { sort: 'timestamp-desc' }),
    );
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(frameDataCalls).toBe(1); // first load fetched frame data once

    // Same entries return → every uuid is cached → no second get_frame_data call.
    await act(async () => {
      await result.current.reload();
    });
    expect(frameDataCalls).toBe(1);
  });
});
