import { useCallback, useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { FrameDataWrapper } from '../components/ResultCard';

/**
 * A single row from the `query_timeline` backend command.
 */
export interface TimelineEntry {
  uuid: string;
  origin: string;
  modality: string;
  timestamp: number | null;
}

export interface UseModalityFramesOptions {
  /** Max entries (after the modality filter) to load frame data for. Undefined = all. */
  limit?: number;
  /** Passed through to `query_timeline`. */
  collectorId?: string;
  /** Passed through to `query_timeline`. */
  textQuery?: string;
  /** Load once on mount (default true). */
  autoLoad?: boolean;
  /** Backend command used to resolve frame data for the keys (default `get_frame_data`). */
  frameCommand?: string;
}

export interface UseModalityFramesResult {
  /** Timeline entries filtered to this modality (newest-first, as the backend returns them). */
  entries: TimelineEntry[];
  /** Frame data for the (limited) entries, in entry order. */
  frames: FrameDataWrapper[];
  loading: boolean;
  error: string | null;
  reload: () => Promise<void>;
}

/**
 * Generalizes the `query_timeline → filter modality → get_frame_data → setState`
 * flow that every modality dashboard used to hand-roll, with the in-flight dedup
 * and per-uuid frame caching that previously only `useReplay` had. A dashboard
 * consumes `{ entries, frames, loading, error, reload }` and maps `frames` to its
 * own domain shape.
 */
export function useModalityFrames(
  modality: string,
  options: UseModalityFramesOptions = {},
): UseModalityFramesResult {
  const { limit, collectorId, textQuery, autoLoad = true, frameCommand = 'get_frame_data' } =
    options;

  const [entries, setEntries] = useState<TimelineEntry[]>([]);
  const [frames, setFrames] = useState<FrameDataWrapper[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // In-flight guard: a reload while one is running is a no-op (dedup).
  const loadInFlight = useRef(false);
  // Per-uuid frame cache so repeated reloads only fetch frames we don't have yet.
  const frameCache = useRef<Map<string, FrameDataWrapper>>(new Map());

  const reload = useCallback(async (): Promise<void> => {
    if (loadInFlight.current) {
      return;
    }
    loadInFlight.current = true;
    setLoading(true);
    setError(null);
    try {
      const timeline = await invoke<TimelineEntry[]>('query_timeline', {
        textQuery,
        collectorId,
      });
      const matched = (Array.isArray(timeline) ? timeline : []).filter(
        (entry) => entry.modality === modality,
      );
      const selected = typeof limit === 'number' ? matched.slice(0, limit) : matched;
      setEntries(selected);

      if (selected.length === 0) {
        setFrames([]);
        return;
      }

      // Only fetch frames we haven't cached yet. Cache by the frame's own uuid
      // (not by request order — `get_frame_data` gives no ordering guarantee).
      const missing = selected.filter((entry) => !frameCache.current.has(entry.uuid));
      if (missing.length > 0) {
        const fetched = await invoke<FrameDataWrapper[]>(frameCommand, {
          keys: missing.map((entry) => ({ uuid: entry.uuid, origin: entry.origin })),
        });
        (Array.isArray(fetched) ? fetched : []).forEach((frame) => {
          if (frame && frame.uuid) {
            frameCache.current.set(frame.uuid, frame);
          }
        });
      }

      // Assemble in entry order, dropping any the backend didn't return.
      const ordered = selected
        .map((entry) => frameCache.current.get(entry.uuid))
        .filter((frame): frame is FrameDataWrapper => frame != null);
      setFrames(ordered);
    } catch (cause) {
      const message = cause instanceof Error ? cause.message : String(cause);
      setError(message);
      setEntries([]);
      setFrames([]);
    } finally {
      loadInFlight.current = false;
      setLoading(false);
    }
  }, [modality, limit, collectorId, textQuery, frameCommand]);

  useEffect(() => {
    if (autoLoad) {
      void reload();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [reload, autoLoad]);

  return { entries, frames, loading, error, reload };
}
