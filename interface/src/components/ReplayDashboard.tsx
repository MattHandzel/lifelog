import { useEffect, useMemo, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Play, RefreshCw, Clock, List, Monitor } from 'lucide-react';
import { Button } from './ui/button';
import { Input } from './ui/input';

interface LifelogDataKeyWrapper {
  uuid: string;
  origin: string;
}

interface ReplayStepWrapper {
  start: number | null;
  end: number | null;
  screen_key: LifelogDataKeyWrapper | null;
  context_keys: LifelogDataKeyWrapper[];
}

interface Screenshot {
  uuid: string;
  timestamp: number | null;
  dataUrl: string;
  width: number;
  height: number;
  mime_type: string;
}

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
    .split(/[,\n]/g)
    .map((s) => s.trim())
    .filter((s) => s.length > 0);
}

export default function ReplayDashboard(): JSX.Element {
  const [screenOrigin, setScreenOrigin] = useState<string>('');
  const [contextOriginsText, setContextOriginsText] = useState<string>('Browser,Ocr');
  const [startDate, setStartDate] = useState<string>('');
  const [endDate, setEndDate] = useState<string>('');
  const [maxSteps, setMaxSteps] = useState<number>(200);
  const [maxContextPerStep, setMaxContextPerStep] = useState<number>(25);
  const [contextPadMs, setContextPadMs] = useState<number>(15000);

  const [isLoading, setIsLoading] = useState(false);
  const [steps, setSteps] = useState<ReplayStepWrapper[]>([]);
  const [selectedIdx, setSelectedIdx] = useState<number>(0);
  const [selectedScreenshot, setSelectedScreenshot] = useState<Screenshot | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Default to "last 10 minutes".
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

  const selectedStep = steps.length > 0 ? steps[Math.min(selectedIdx, steps.length - 1)] : null;

  async function loadReplay(): Promise<void> {
    setIsLoading(true);
    setError(null);
    setSelectedScreenshot(null);
    try {
      const startTime = toUnixSeconds(startDate);
      const endTime = toUnixSeconds(endDate);
      if (startTime == null || endTime == null) {
        throw new Error('Start and end must be set');
      }
      if (startTime >= endTime) {
        throw new Error('Start must be before end');
      }

      const res = await invoke<ReplayStepWrapper[]>('replay', {
        screenOrigin: screenOrigin || undefined,
        contextOrigins: contextOrigins.length > 0 ? contextOrigins : undefined,
        startTime,
        endTime,
        maxSteps,
        maxContextPerStep,
        contextPadMs,
      });

      setSteps(Array.isArray(res) ? res : []);
      setSelectedIdx(0);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      console.error('[ReplayDashboard] replay failed:', e);
      setSteps([]);
      setError(msg);
    } finally {
      setIsLoading(false);
    }
  }

  async function loadScreenshotForStep(step: ReplayStepWrapper | null): Promise<void> {
    setSelectedScreenshot(null);
    if (!step?.screen_key) return;
    try {
      const frames = await invoke<Screenshot[]>('get_screenshots_data', { keys: [step.screen_key] });
      setSelectedScreenshot(frames?.[0] ?? null);
    } catch (e) {
      console.error('[ReplayDashboard] get_screenshots_data failed:', e);
      setSelectedScreenshot(null);
    }
  }

  useEffect(() => {
    // Auto-load the first step's screenshot once steps load.
    void loadScreenshotForStep(selectedStep);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedIdx, steps.length]);

  const stepsSummary = useMemo(() => {
    if (steps.length === 0) return 'No steps loaded';
    const first = steps[0];
    const last = steps[steps.length - 1];
    return `${steps.length} steps (${fmtUnixSeconds(first?.start)} -> ${fmtUnixSeconds(last?.end)})`;
  }, [steps]);

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
            <Button onClick={loadReplay} disabled={isLoading}>
              {isLoading ? (
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

          {error && (
            <div className="mt-2 text-sm text-red-300">
              {error}
            </div>
          )}
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
                {steps.map((s, idx) => (
                  <button
                    key={`${s.screen_key?.uuid ?? 'no-screen'}:${idx}`}
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
                      <div className="text-sm text-[#F9FAFB] truncate">
                        {fmtUnixSeconds(s.start)}
                      </div>
                      <div className="text-xs text-[#9CA3AF]">
                        {s.context_keys?.length ?? 0} ctx
                      </div>
                    </div>
                    <div className="text-xs text-[#9CA3AF] mt-1 truncate">
                      {s.screen_key ? `${s.screen_key.origin} • ${s.screen_key.uuid.slice(0, 8)}…` : 'No screen key'}
                    </div>
                  </button>
                ))}
              </div>
            )}
          </div>
        </div>

        <div className="card lg:col-span-2">
          <div className="p-6 space-y-4">
            <div className="flex items-center justify-between gap-4">
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

            <div className="rounded-lg border border-[#232B3D] bg-[#0F111A] overflow-hidden">
              {selectedScreenshot ? (
                <img
                  src={selectedScreenshot.dataUrl}
                  alt="Replay step screenshot"
                  className="w-full h-auto block"
                />
              ) : (
                <div className="flex flex-col items-center justify-center py-16 text-[#9CA3AF]">
                  <Monitor className="w-12 h-12 mb-3" />
                  <div className="text-sm">No screenshot loaded for this step</div>
                </div>
              )}
            </div>

            <div className="rounded-lg border border-[#232B3D] bg-[#1A1E2E] p-4">
              <div className="text-sm font-medium text-[#F9FAFB] mb-2">Context Keys</div>
              {selectedStep?.context_keys?.length ? (
                <div className="space-y-1">
                  {selectedStep.context_keys.slice(0, 50).map((k, i) => (
                    <div key={`${k.uuid}:${i}`} className="text-xs text-[#9CA3AF] truncate">
                      {k.origin} • {k.uuid}
                    </div>
                  ))}
                </div>
              ) : (
                <div className="text-sm text-[#9CA3AF]">No context for this step</div>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

