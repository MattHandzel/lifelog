import { Clipboard, FileText, Keyboard, Monitor, Terminal } from 'lucide-react';
import type { FrameDataWrapper } from './ResultCard';
import type { ReplayStepWrapper, Screenshot } from '../lib/useReplay';

interface ReplayFrameProps {
  step: ReplayStepWrapper | null;
  screenshot: Screenshot | null;
  contextFrames: FrameDataWrapper[];
  isLoadingContext: boolean;
}

interface ActiveWindow {
  application: string;
  windowTitle: string;
}

function normalize(value: string | null | undefined): string {
  return (value ?? '').trim().toLowerCase();
}

function byTimestampAsc(a: FrameDataWrapper, b: FrameDataWrapper): number {
  return (a.timestamp ?? 0) - (b.timestamp ?? 0);
}

function deriveActiveWindow(frames: FrameDataWrapper[]): ActiveWindow | null {
  const windowFrames = frames
    .filter((frame) => frame.modality.toLowerCase() === 'windowactivity')
    .sort(byTimestampAsc);
  if (windowFrames.length === 0) return null;
  const latest = windowFrames[windowFrames.length - 1];
  const application = latest.application?.trim() ?? '';
  const windowTitle = latest.window_title?.trim() ?? '';
  if (!application && !windowTitle) return null;
  return { application, windowTitle };
}

function filterKeystrokes(frames: FrameDataWrapper[], activeWindow: ActiveWindow | null): FrameDataWrapper[] {
  const keyFrames = frames.filter((frame) => frame.modality.toLowerCase() === 'keystrokes').sort(byTimestampAsc);
  if (!activeWindow) return keyFrames;

  const activeApp = normalize(activeWindow.application);
  const activeTitle = normalize(activeWindow.windowTitle);

  return keyFrames.filter((frame) => {
    const frameApp = normalize(frame.application);
    const frameTitle = normalize(frame.window_title);
    if (frameApp && activeApp && frameApp === activeApp) return true;
    if (frameTitle && activeTitle && frameTitle === activeTitle) return true;
    return false;
  });
}

export default function ReplayFrame({ step, screenshot, contextFrames, isLoadingContext }: ReplayFrameProps): JSX.Element {
  const activeWindow = deriveActiveWindow(contextFrames);
  const keyFrames = filterKeystrokes(contextFrames, activeWindow).slice(-5);
  const clipboardFrames = contextFrames
    .filter((frame) => frame.modality.toLowerCase() === 'clipboard')
    .sort(byTimestampAsc)
    .slice(-3);
  const commandFrames = contextFrames
    .filter((frame) => frame.modality.toLowerCase() === 'shellhistory')
    .sort(byTimestampAsc)
    .slice(-3);
  const audioFrames = contextFrames.filter((frame) => frame.modality.toLowerCase() === 'audio' && frame.audio_data_url);

  return (
    <div className="space-y-4">
      <div className="relative rounded-lg border border-[#232B3D] bg-[#0F111A] overflow-hidden min-h-[280px]">
        {screenshot ? (
          <img src={screenshot.dataUrl} alt="Replay step screenshot" className="w-full h-auto block" />
        ) : (
          <div className="flex flex-col items-center justify-center py-16 text-[#9CA3AF]">
            <Monitor className="w-12 h-12 mb-3" />
            <div className="text-sm">No screenshot loaded for this step</div>
          </div>
        )}

        <div className="absolute inset-0 pointer-events-none p-3 flex flex-col justify-between gap-3">
          <div className="flex items-start justify-between gap-3">
            <div className="max-w-[60%] rounded-md bg-[#0B1020]/75 border border-[#334155] px-3 py-2 text-xs text-[#E5E7EB] backdrop-blur-sm">
              <div className="flex items-center gap-2 mb-1 text-[#9CA3AF]">
                <Monitor className="w-3.5 h-3.5" />
                Active window
              </div>
              {activeWindow ? (
                <>
                  <div className="font-medium text-[#F9FAFB] truncate">{activeWindow.windowTitle || 'Untitled window'}</div>
                  {activeWindow.application && <div className="truncate text-[#9CA3AF]">{activeWindow.application}</div>}
                </>
              ) : (
                <div className="text-[#9CA3AF]">No window metadata in this step</div>
              )}
            </div>

            <div className="max-w-[45%] rounded-md bg-[#0B1020]/75 border border-[#334155] px-3 py-2 text-xs text-[#E5E7EB] backdrop-blur-sm">
              <div className="flex items-center gap-2 mb-1 text-[#9CA3AF]">
                <Keyboard className="w-3.5 h-3.5" />
                Keystrokes
              </div>
              {keyFrames.length > 0 ? (
                <div className="space-y-0.5">
                  {keyFrames.map((frame) => (
                    <div key={frame.uuid} className="truncate text-[#F9FAFB]">{frame.text || '[empty]'}</div>
                  ))}
                </div>
              ) : (
                <div className="text-[#9CA3AF]">No keystrokes for active window</div>
              )}
            </div>
          </div>

          <div className="flex items-end justify-between gap-3">
            <div className="max-w-[55%] rounded-md bg-[#0B1020]/75 border border-[#334155] px-3 py-2 text-xs text-[#E5E7EB] backdrop-blur-sm">
              <div className="flex items-center gap-2 mb-1 text-[#9CA3AF]">
                <Clipboard className="w-3.5 h-3.5" />
                Clipboard
              </div>
              {clipboardFrames.length > 0 ? (
                <div className="space-y-0.5">
                  {clipboardFrames.map((frame) => (
                    <div key={frame.uuid} className="line-clamp-2 text-[#F9FAFB]">{frame.text || '[empty]'}</div>
                  ))}
                </div>
              ) : (
                <div className="text-[#9CA3AF]">No clipboard events</div>
              )}
            </div>

            <div className="max-w-[45%] rounded-md bg-[#0B1020]/75 border border-[#334155] px-3 py-2 text-xs text-[#E5E7EB] backdrop-blur-sm">
              <div className="flex items-center gap-2 mb-1 text-[#9CA3AF]">
                <Terminal className="w-3.5 h-3.5" />
                Commands
              </div>
              {commandFrames.length > 0 ? (
                <div className="space-y-0.5">
                  {commandFrames.map((frame) => (
                    <div key={frame.uuid} className="truncate text-[#F9FAFB]">{frame.command || '[empty]'}</div>
                  ))}
                </div>
              ) : (
                <div className="text-[#9CA3AF]">No commands</div>
              )}
            </div>
          </div>
        </div>
      </div>

      {audioFrames.length > 0 && (
        <div className="rounded-lg border border-[#232B3D] bg-[#1A1E2E] p-4 space-y-2">
          <div className="flex items-center gap-2 text-sm font-medium text-[#F9FAFB]">
            <FileText className="w-4 h-4 text-cyan-400" />
            Audio ({audioFrames.length})
          </div>
          {audioFrames.map((frame) => (
            <audio key={frame.uuid} controls src={frame.audio_data_url ?? undefined} className="w-full h-8" />
          ))}
        </div>
      )}

      {isLoadingContext && (
        <div className="text-xs text-[#9CA3AF]">Loading context overlays...</div>
      )}

      <div className="rounded-lg border border-[#232B3D] bg-[#1A1E2E] p-4">
        <div className="text-sm font-medium text-[#F9FAFB] mb-2">Context Keys</div>
        {step?.context_keys?.length ? (
          <div className="space-y-1 max-h-36 overflow-y-auto pr-1">
            {step.context_keys.slice(0, 100).map((key, index) => (
              <div key={`${key.uuid}:${index}`} className="text-xs text-[#9CA3AF] truncate">
                {key.origin} • {key.uuid}
              </div>
            ))}
          </div>
        ) : (
          <div className="text-sm text-[#9CA3AF]">No context for this step</div>
        )}
      </div>
    </div>
  );
}
