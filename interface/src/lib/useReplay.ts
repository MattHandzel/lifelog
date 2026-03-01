import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { FrameDataWrapper } from '../components/ResultCard';

export interface LifelogDataKeyWrapper {
  uuid: string;
  origin: string;
}

export interface ReplayStepWrapper {
  start: number | null;
  end: number | null;
  screen_key: LifelogDataKeyWrapper | null;
  context_keys: LifelogDataKeyWrapper[];
}

export interface Screenshot {
  uuid: string;
  timestamp: number | null;
  dataUrl: string;
  width: number;
  height: number;
  mime_type: string;
}

export interface LoadReplayArgs {
  screenOrigin?: string;
  contextOrigins?: string[];
  startTime: number;
  endTime: number;
  maxSteps?: number;
  maxContextPerStep?: number;
  contextPadMs?: number;
}

interface UseReplayState {
  steps: ReplayStepWrapper[];
  selectedIdx: number;
  selectedStep: ReplayStepWrapper | null;
  selectedScreenshot: Screenshot | null;
  selectedContext: FrameDataWrapper[];
  isLoadingReplay: boolean;
  isLoadingContext: boolean;
  isPlaying: boolean;
  playbackMs: number;
  bufferedScreenshots: number;
  totalScreenSteps: number;
  isPrefetching: boolean;
  error: string | null;
  backgroundError: string | null;
  loadReplay: (args: LoadReplayArgs) => Promise<void>;
  setSelectedIdx: (idx: number) => void;
  stepForward: () => void;
  stepBackward: () => void;
  togglePlay: () => void;
  setPlaybackMs: (value: number) => void;
}

const PREFETCH_AHEAD = 18;
const PREFETCH_BATCH = 6;

export function useReplay(): UseReplayState {
  const [steps, setSteps] = useState<ReplayStepWrapper[]>([]);
  const [selectedIdx, setSelectedIdxState] = useState(0);
  const [isLoadingReplay, setIsLoadingReplay] = useState(false);
  const [isLoadingContext, setIsLoadingContext] = useState(false);
  const [isPlaying, setIsPlaying] = useState(false);
  const [playbackMs, setPlaybackMs] = useState(800);
  const [error, setError] = useState<string | null>(null);
  const [backgroundError, setBackgroundError] = useState<string | null>(null);

  const [screensByUuid, setScreensByUuid] = useState<Record<string, Screenshot>>({});
  const [contextByStep, setContextByStep] = useState<Record<number, FrameDataWrapper[]>>({});
  const [isPrefetching, setIsPrefetching] = useState(false);

  const screenFetchInFlight = useRef<Set<string>>(new Set());
  const contextFetchInFlight = useRef<Set<number>>(new Set());

  const selectedStep = steps.length > 0 ? steps[Math.min(selectedIdx, steps.length - 1)] : null;
  const selectedScreenUuid = selectedStep?.screen_key?.uuid ?? null;
  const selectedScreenshot = selectedScreenUuid ? screensByUuid[selectedScreenUuid] ?? null : null;
  const selectedContext = contextByStep[selectedIdx] ?? [];

  const totalScreenSteps = useMemo(
    () => steps.reduce((count, step) => (step.screen_key ? count + 1 : count), 0),
    [steps],
  );

  const bufferedScreenshots = useMemo(() => {
    if (totalScreenSteps === 0) return 0;
    const required = new Set(steps.map((step) => step.screen_key?.uuid).filter(Boolean) as string[]);
    let cached = 0;
    required.forEach((uuid) => {
      if (screensByUuid[uuid]) cached += 1;
    });
    return cached;
  }, [screensByUuid, steps, totalScreenSteps]);

  const setSelectedIdx = useCallback((idx: number) => {
    setSelectedIdxState((current) => {
      if (steps.length === 0) return 0;
      const clamped = Math.min(Math.max(idx, 0), steps.length - 1);
      if (clamped !== current) {
        setIsPlaying(false);
      }
      return clamped;
    });
  }, [steps.length]);

  const stepForward = useCallback(() => {
    setSelectedIdxState((current) => {
      const next = Math.min(current + 1, Math.max(steps.length - 1, 0));
      if (next >= Math.max(steps.length - 1, 0)) {
        setIsPlaying(false);
      }
      return next;
    });
  }, [steps.length]);

  const stepBackward = useCallback(() => {
    setIsPlaying(false);
    setSelectedIdxState((current) => Math.max(current - 1, 0));
  }, []);

  const togglePlay = useCallback(() => {
    if (steps.length === 0) return;
    setIsPlaying((playing) => !playing);
  }, [steps.length]);

  const loadReplay = useCallback(async (args: LoadReplayArgs): Promise<void> => {
    setIsLoadingReplay(true);
    setError(null);
    setBackgroundError(null);
    setIsPlaying(false);
    setScreensByUuid({});
    setContextByStep({});
    screenFetchInFlight.current.clear();
    contextFetchInFlight.current.clear();

    try {
      const response = await invoke<ReplayStepWrapper[]>('replay', {
        screenOrigin: args.screenOrigin,
        contextOrigins: args.contextOrigins,
        startTime: args.startTime,
        endTime: args.endTime,
        maxSteps: args.maxSteps,
        maxContextPerStep: args.maxContextPerStep,
        contextPadMs: args.contextPadMs,
      });
      setSteps(Array.isArray(response) ? response : []);
      setSelectedIdxState(0);
    } catch (cause) {
      const message = cause instanceof Error ? cause.message : String(cause);
      setError(message);
      setSteps([]);
      setSelectedIdxState(0);
    } finally {
      setIsLoadingReplay(false);
    }
  }, []);

  useEffect(() => {
    if (!selectedStep?.context_keys?.length) {
      return;
    }
    if (contextByStep[selectedIdx] || contextFetchInFlight.current.has(selectedIdx)) {
      return;
    }

    contextFetchInFlight.current.add(selectedIdx);
    setIsLoadingContext(true);
    void invoke<FrameDataWrapper[]>('get_frame_data', { keys: selectedStep.context_keys })
      .then((frames) => {
        setContextByStep((current) => ({
          ...current,
          [selectedIdx]: Array.isArray(frames) ? frames : [],
        }));
      })
      .catch((cause) => {
        const message = cause instanceof Error ? cause.message : String(cause);
        setBackgroundError(`Context load failed: ${message}`);
      })
      .finally(() => {
        contextFetchInFlight.current.delete(selectedIdx);
        setIsLoadingContext(false);
      });
  }, [contextByStep, selectedIdx, selectedStep]);

  useEffect(() => {
    if (steps.length === 0) return;
    const targets = steps
      .slice(selectedIdx, selectedIdx + PREFETCH_AHEAD)
      .map((step) => step.screen_key)
      .filter((key): key is LifelogDataKeyWrapper => key !== null)
      .filter((key) => !screensByUuid[key.uuid] && !screenFetchInFlight.current.has(key.uuid));

    if (targets.length === 0) {
      return;
    }

    const batch = targets.slice(0, PREFETCH_BATCH);
    batch.forEach((key) => screenFetchInFlight.current.add(key.uuid));
    setIsPrefetching(true);

    void invoke<Screenshot[]>('get_screenshots_data', { keys: batch })
      .then((screens) => {
        if (!Array.isArray(screens) || screens.length === 0) {
          return;
        }
        setScreensByUuid((current) => {
          const next = { ...current };
          screens.forEach((screen) => {
            next[screen.uuid] = screen;
          });
          return next;
        });
      })
      .catch((cause) => {
        const message = cause instanceof Error ? cause.message : String(cause);
        setBackgroundError(`Screenshot prefetch failed: ${message}`);
      })
      .finally(() => {
        batch.forEach((key) => screenFetchInFlight.current.delete(key.uuid));
        setIsPrefetching(false);
      });
  }, [screensByUuid, selectedIdx, steps]);

  useEffect(() => {
    if (!isPlaying || steps.length === 0) return;
    const timer = window.setInterval(() => {
      setSelectedIdxState((current) => {
        if (current >= steps.length - 1) {
          setIsPlaying(false);
          return current;
        }
        return current + 1;
      });
    }, playbackMs);
    return () => window.clearInterval(timer);
  }, [isPlaying, playbackMs, steps.length]);

  return {
    steps,
    selectedIdx,
    selectedStep,
    selectedScreenshot,
    selectedContext,
    isLoadingReplay,
    isLoadingContext,
    isPlaying,
    playbackMs,
    bufferedScreenshots,
    totalScreenSteps,
    isPrefetching,
    error,
    backgroundError,
    loadReplay,
    setSelectedIdx,
    stepForward,
    stepBackward,
    togglePlay,
    setPlaybackMs,
  };
}
