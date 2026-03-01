import { useEffect, useMemo, useState } from 'react';
import { Clock, List, Monitor, Pause, Play, RefreshCw, SkipBack, SkipForward } from 'lucide-react';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { Slider } from './ui/slider';
import ReplayFrame from './ReplayFrame';
import { useReplay } from '../lib/useReplay';

function toUnixSeconds(dtLocal: string): number | null {
  if (!dtLocal) return null;
  const ms = new Date(dtLocal).getTime();
  if (!Number.isFinite(ms)) return null;
  return Math.floor(ms / 1000);
}

function fmtUnixSeconds(ts: number | null | undefined): string {
  if (!ts) return 'N/A';
  return new Date(ts * 1000).toLocaleString();
}

function parseOrigins(text: string): string[] {
  return text
    .split(/[\n,]/g)
    .map((s) => s.trim())
    .filter((s) => s.length > 0);
}

export default function ReplayDashboard(): JSX.Element {
  const [screenOrigin, setScreenOrigin] = useState<string>('');
  const [contextOriginsText, setContextOriginsText] = useState<string>('Browser,Ocr,Clipboard,Keystrokes,ShellHistory,WindowActivity,Audio');
  const [startDate, setStartDate] = useState<string>('');
  const [endDate, setEndDate] = useState<string>('');
  const [maxSteps, setMaxSteps] = useState<number>(200);
  const [maxContextPerStep, setMaxContextPerStep] = useState<number>(25);
  const [contextPadMs, setContextPadMs] = useState<number>(15000);

  const {
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
  } = useReplay();

  useEffect(() => {
    const now = new Date();
    const tenMinAgo = new Date(now.getTime() - 10 * 60 * 1000);
    const toLocal = (d: Date): string => {
      const pad = (n: number) => String(n).padStart(2, '0');
      return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
    };
    setEndDate(toLocal(now));
    setStartDate(toLocal(tenMinAgo));
  }, []);

  const contextOrigins = useMemo(() => parseOrigins(contextOriginsText), [contextOriginsText]);

  const stepsSummary = useMemo(() => {
    if (steps.length === 0) return 'No steps loaded';
    const first = steps[0];
    const last = steps[steps.length - 1];
    return `${steps.length} steps (${fmtUnixSeconds(first?.start)} -> ${fmtUnixSeconds(last?.end)})`;
  }, [steps]);

  const bufferSummary = useMemo(() => {
    if (totalScreenSteps === 0) return 'No screen frames in replay';
    return `${bufferedScreenshots}/${totalScreenSteps} frames buffered`;
  }, [bufferedScreenshots, totalScreenSteps]);

  async function onLoadReplay(): Promise<void> {
    const startTime = toUnixSeconds(startDate);
    const endTime = toUnixSeconds(endDate);
    if (startTime == null || endTime == null) {
      return;
    }
    if (startTime >= endTime) {
      return;
    }

    await loadReplay({
      screenOrigin: screenOrigin || undefined,
      contextOrigins: contextOrigins.length > 0 ? contextOrigins : undefined,
      startTime,
      endTime,
      maxSteps,
      maxContextPerStep,
      contextPadMs,
    });
  }

  return (
    <div className="p-6 md:p-8 space-y-6">
      <div className="mb-2">
        <div className="flex items-center gap-3 mb-2">
          <Play className="w-8 h-8 text-[#4C8BF5]" />
          <h1 className="title">Replay</h1>
        </div>
        <p className="subtitle">Step through a time window and see aligned context across streams</p>
      </div>

      <div className="card">
        <div className="p-6 space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="space-y-2">
              <label className="block text-sm font-medium text-[#F9FAFB]">Start</label>
              <Input
                type="datetime-local"
                value={startDate}
                onChange={(e) => setStartDate(e.target.value)}
                className="bg-[#1C2233] border-[#232B3D] text-[#F9FAFB]"
              />
            </div>
            <div className="space-y-2">
              <label className="block text-sm font-medium text-[#F9FAFB]">End</label>
              <Input
                type="datetime-local"
                value={endDate}
                onChange={(e) => setEndDate(e.target.value)}
                className="bg-[#1C2233] border-[#232B3D] text-[#F9FAFB]"
              />
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="space-y-2">
              <label className="block text-sm font-medium text-[#F9FAFB]">Screen Origin (optional)</label>
              <Input
                type="text"
                placeholder='e.g. "laptop:Screen" (leave blank to auto-pick)'
                value={screenOrigin}
                onChange={(e) => setScreenOrigin(e.target.value)}
                className="bg-[#1C2233] border-[#232B3D] text-[#F9FAFB]"
              />
            </div>
            <div className="space-y-2">
              <label className="block text-sm font-medium text-[#F9FAFB]">Context Origins</label>
              <Input
                type="text"
                placeholder='e.g. "Browser,Ocr,Clipboard" or "*"'
                value={contextOriginsText}
                onChange={(e) => setContextOriginsText(e.target.value)}
                className="bg-[#1C2233] border-[#232B3D] text-[#F9FAFB]"
              />
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="space-y-2">
              <label className="block text-sm font-medium text-[#F9FAFB]">Max Steps</label>
              <Input
                type="number"
                min={1}
                max={5000}
                value={maxSteps}
                onChange={(e) => setMaxSteps(Number(e.target.value))}
                className="bg-[#1C2233] border-[#232B3D] text-[#F9FAFB]"
              />
            </div>
            <div className="space-y-2">
              <label className="block text-sm font-medium text-[#F9FAFB]">Max Context/Step</label>
              <Input
                type="number"
                min={0}
                max={1000}
                value={maxContextPerStep}
                onChange={(e) => setMaxContextPerStep(Number(e.target.value))}
                className="bg-[#1C2233] border-[#232B3D] text-[#F9FAFB]"
              />
            </div>
            <div className="space-y-2">
              <label className="block text-sm font-medium text-[#F9FAFB]">Context Pad (ms)</label>
              <Input
                type="number"
                min={0}
                max={300000}
                value={contextPadMs}
                onChange={(e) => setContextPadMs(Number(e.target.value))}
                className="bg-[#1C2233] border-[#232B3D] text-[#F9FAFB]"
              />
            </div>
          </div>

          <div className="flex gap-2 justify-end border-t border-[#232B3D] pt-4">
            <Button onClick={onLoadReplay} disabled={isLoadingReplay}>
              {isLoadingReplay ? (
                <>
                  <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
                  Loading...
                </>
              ) : (
                <>
                  <Play className="w-4 h-4 mr-2" />
                  Load Replay
                </>
              )}
            </Button>
          </div>

          {error && <div className="mt-2 text-sm text-red-300">{error}</div>}
          {backgroundError && <div className="mt-1 text-xs text-amber-300">{backgroundError}</div>}
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="card lg:col-span-1">
          <div className="p-6">
            <div className="flex items-center gap-2 mb-3">
              <List className="w-5 h-5 text-[#9CA3AF]" />
              <div className="text-sm text-[#9CA3AF]">{stepsSummary}</div>
            </div>

            {steps.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-10 text-center text-[#9CA3AF]">
                <Clock className="w-10 h-10 mb-3" />
                <div className="text-[#F9FAFB] font-medium mb-1">No replay loaded</div>
                <div className="text-sm">Choose a window and click Load Replay</div>
              </div>
            ) : (
              <div className="space-y-2 max-h-[520px] overflow-auto pr-1">
                {steps.map((step, idx) => (
                  <button
                    key={`${step.screen_key?.uuid ?? 'no-screen'}:${idx}`}
                    type="button"
                    className={[
                      'w-full text-left p-3 rounded-lg border transition-colors',
                      idx === selectedIdx
                        ? 'bg-[#232B3D] border-[#4C8BF5]/50'
                        : 'bg-[#1A1E2E] border-[#232B3D] hover:bg-[#232B3D]',
                    ].join(' ')}
                    onClick={() => setSelectedIdx(idx)}
                  >
                    <div className="flex items-center justify-between gap-2">
                      <div className="text-sm text-[#F9FAFB] truncate">{fmtUnixSeconds(step.start)}</div>
                      <div className="text-xs text-[#9CA3AF]">{step.context_keys?.length ?? 0} ctx</div>
                    </div>
                    <div className="text-xs text-[#9CA3AF] mt-1 truncate">
                      {step.screen_key ? `${step.screen_key.origin} • ${step.screen_key.uuid.slice(0, 8)}...` : 'No screen key'}
                    </div>
                  </button>
                ))}
              </div>
            )}
          </div>
        </div>

        <div className="card lg:col-span-2">
          <div className="p-6 space-y-4">
            <div className="flex items-center justify-between gap-4 flex-wrap">
              <div className="flex items-center gap-2">
                <Monitor className="w-5 h-5 text-[#9CA3AF]" />
                <div className="text-sm text-[#9CA3AF]">
                  Step {steps.length === 0 ? 0 : selectedIdx + 1} / {steps.length}
                </div>
              </div>
              <div className="text-xs text-[#9CA3AF]">
                {selectedStep ? `${fmtUnixSeconds(selectedStep.start)} -> ${fmtUnixSeconds(selectedStep.end)}` : ''}
              </div>
            </div>

            <div className="rounded-lg border border-[#232B3D] bg-[#1A1E2E] p-4 space-y-4">
              <div className="flex items-center gap-2 flex-wrap">
                <Button
                  variant="outline"
                  size="icon"
                  aria-label="Previous step"
                  onClick={stepBackward}
                  disabled={steps.length === 0 || selectedIdx === 0}
                  className="bg-[#0F111A] border-[#334155]"
                >
                  <SkipBack className="w-4 h-4" />
                </Button>

                <Button
                  variant="outline"
                  size="icon"
                  aria-label={isPlaying ? 'Pause replay' : 'Play replay'}
                  onClick={togglePlay}
                  disabled={steps.length === 0}
                  className="bg-[#0F111A] border-[#334155]"
                >
                  {isPlaying ? <Pause className="w-4 h-4" /> : <Play className="w-4 h-4" />}
                </Button>

                <Button
                  variant="outline"
                  size="icon"
                  aria-label="Next step"
                  onClick={stepForward}
                  disabled={steps.length === 0 || selectedIdx >= steps.length - 1}
                  className="bg-[#0F111A] border-[#334155]"
                >
                  <SkipForward className="w-4 h-4" />
                </Button>

                <div className="ml-2 text-xs text-[#9CA3AF]">Playback</div>
                <select
                  value={playbackMs}
                  onChange={(e) => setPlaybackMs(Number(e.target.value))}
                  className="h-9 px-2 rounded-md border border-[#334155] bg-[#0F111A] text-[#E5E7EB] text-xs"
                  aria-label="Playback speed"
                >
                  <option value={300}>Fast (300ms)</option>
                  <option value={500}>Quick (500ms)</option>
                  <option value={800}>Normal (800ms)</option>
                  <option value={1200}>Slow (1200ms)</option>
                </select>

                <div className="ml-auto text-xs text-[#9CA3AF]">
                  {bufferSummary}{isPrefetching ? ' • prefetching' : ''}
                </div>
              </div>

              {steps.length > 1 ? (
                <Slider
                  value={[selectedIdx]}
                  min={0}
                  max={steps.length - 1}
                  step={1}
                  onValueChange={(value) => setSelectedIdx(value[0] ?? 0)}
                  aria-label="Replay timeline scrubber"
                />
              ) : (
                <div className="h-2 rounded-full bg-[#2a3142]" />
              )}
            </div>

            <ReplayFrame
              step={selectedStep}
              screenshot={selectedScreenshot}
              contextFrames={selectedContext}
              isLoadingContext={isLoadingContext}
            />
          </div>
        </div>
      </div>
    </div>
  );
}
