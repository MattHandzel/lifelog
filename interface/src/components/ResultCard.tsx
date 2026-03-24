import { useEffect, useRef, useState } from 'react';
import { Image, FileAudio, File, Mouse, Cloud, Monitor, Cpu, Clipboard, Globe, Terminal, Activity, MessageSquareText, Play } from 'lucide-react';
import { Card, CardContent } from './ui/card';

export interface SearchResult {
  id: string;
  type: 'image' | 'audio' | 'file';
  name: string;
  path: string;
  timestamp: number;
  source: string;
  modality: string;
  preview?: string;
  size?: number;
  duration?: number;
  metadata?: Record<string, unknown>;
  snippet?: string;
  highlightTerms?: string[];
}

export interface ProcessInfoWrapper {
  pid: number;
  ppid: number;
  name: string;
  exe: string;
  cmdline: string;
  status: string;
  cpu_usage: number;
  memory_usage: number;
  threads: number;
  user: string;
  start_time: number;
}

export interface FrameDataWrapper {
  uuid: string;
  modality: string;
  timestamp: number | null;
  text: string | null;
  url: string | null;
  title: string | null;
  visit_count: number | null;
  command: string | null;
  working_dir: string | null;
  exit_code: number | null;
  application: string | null;
  window_title: string | null;
  duration_secs: number | null;
  audio_data_url: string | null;
  codec: string | null;
  sample_rate: number | null;
  channels: number | null;
  audio_duration_secs: number | null;
  image_data_url: string | null;
  width: number | null;
  height: number | null;
  mime_type: string | null;
  camera_device: string | null;
  processes: ProcessInfoWrapper[] | null;
  transcription_model: string | null;
  transcription_confidence: number | null;
  source_frame_uuid: string | null;
}

function formatDate(timestamp: number | null): string {
  if (timestamp == null || timestamp === 0) return 'No timestamp';
  const ms = timestamp < 1e12 ? timestamp * 1000 : timestamp;
  return new Date(ms).toLocaleString();
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}

function displayNameFromUuid(uuid: string): string {
  return uuid.length > 16 ? `${uuid.slice(0, 16)}...` : uuid;
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function renderHighlightedSnippet(snippet: string, terms: string[]): JSX.Element {
  const cleanedTerms = Array.from(new Set(terms.map((t) => t.trim().toLowerCase()).filter(Boolean)));
  if (cleanedTerms.length === 0) {
    return <>{snippet}</>;
  }
  const matcher = new RegExp(`(${cleanedTerms.map(escapeRegExp).join('|')})`, 'ig');
  const parts = snippet.split(matcher);
  const termSet = new Set(cleanedTerms);
  return (
    <>
      {parts.map((part, index) => (
        termSet.has(part.toLowerCase())
          ? <mark key={`${part}-${index}`} className="bg-[#4C8BF5]/30 text-[#F9FAFB] rounded px-0.5">{part}</mark>
          : <span key={`${part}-${index}`}>{part}</span>
      ))}
    </>
  );
}

function Thumbnail({ src, alt }: { src: string; alt: string }): JSX.Element {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const [visible, setVisible] = useState(false);
  const [loaded, setLoaded] = useState(false);

  useEffect(() => {
    const node = containerRef.current;
    if (!node) {
      return;
    }
    if (typeof IntersectionObserver === 'undefined') {
      setVisible(true);
      return;
    }
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries.some((entry) => entry.isIntersecting)) {
          setVisible(true);
          observer.disconnect();
        }
      },
      { rootMargin: '200px 0px' },
    );
    observer.observe(node);
    return () => observer.disconnect();
  }, []);

  return (
    <div ref={containerRef} className="aspect-video bg-[#0F111A] relative overflow-hidden">
      {visible ? (
        <>
          <img
            src={src}
            alt={alt}
            loading="lazy"
            onLoad={() => setLoaded(true)}
            className={`w-full h-full object-cover transition-opacity duration-200 ${loaded ? 'opacity-100' : 'opacity-0'}`}
          />
          {!loaded && <div className="absolute inset-0 animate-pulse bg-[#151a28]" />}
        </>
      ) : (
        <div className="absolute inset-0 animate-pulse bg-[#151a28]" />
      )}
    </div>
  );
}

function ModalityIcon({ modality }: { modality: string }): JSX.Element {
  const lower = modality.toLowerCase();
  if (lower === 'screen' || lower === 'camera') return <Image className="w-5 h-5 text-blue-400" />;
  if (lower === 'audio' || lower === 'microphone') return <FileAudio className="w-5 h-5 text-green-400" />;
  if (lower === 'mouse') return <Mouse className="w-5 h-5 text-purple-400" />;
  if (lower === 'weather') return <Cloud className="w-5 h-5 text-sky-400" />;
  if (lower === 'hyprland' || lower === 'windowactivity') return <Monitor className="w-5 h-5 text-indigo-400" />;
  if (lower === 'processes') return <Cpu className="w-5 h-5 text-orange-400" />;
  if (lower === 'clipboard') return <Clipboard className="w-5 h-5 text-yellow-400" />;
  if (lower === 'browser') return <Globe className="w-5 h-5 text-cyan-400" />;
  if (lower === 'shellhistory' || lower === 'shell_history') return <Terminal className="w-5 h-5 text-rose-400" />;
  if (lower === 'keystroke' || lower === 'keystrokes') return <Activity className="w-5 h-5 text-pink-400" />;
  if (lower === 'transcription') return <MessageSquareText className="w-5 h-5 text-emerald-400" />;
  return <File className="w-5 h-5 text-gray-400" />;
}

function ModalityPreview({ result }: { result: SearchResult }): JSX.Element | null {
  const lower = result.modality.toLowerCase();

  if ((lower === 'screen' || lower === 'camera') && result.preview) {
    return (
      <Thumbnail src={result.preview} alt={result.name} />
    );
  }

  if (lower === 'audio' || lower === 'microphone') {
    return (
      <div className="aspect-video bg-[#0F111A] flex flex-col items-center justify-center gap-2">
        <FileAudio className="w-12 h-12 text-[#4C8BF5]" />
        {result.duration != null && (
          <span className="text-xs text-[#9CA3AF]">
            {Math.floor(result.duration / 60)}:{(result.duration % 60).toString().padStart(2, '0')}
          </span>
        )}
      </div>
    );
  }

  if (lower === 'transcription') {
    const text = result.snippet || (result.metadata?.text as string) || '';
    return (
      <div className="aspect-video bg-[#0F111A] flex flex-col items-start justify-start p-3 gap-1 overflow-hidden">
        <MessageSquareText className="w-6 h-6 text-emerald-400 shrink-0" />
        <p className="text-xs text-[#D1D5DB] line-clamp-4 leading-relaxed">{text}</p>
      </div>
    );
  }

  if (lower === 'ocr') {
    const text = result.snippet || (result.metadata?.text as string) || '';
    if (text) {
      return (
        <div className="aspect-video bg-[#0F111A] flex flex-col items-start justify-start p-3 gap-1 overflow-hidden">
          <File className="w-6 h-6 text-blue-400 shrink-0" />
          <p className="text-xs text-[#D1D5DB] line-clamp-4 leading-relaxed font-mono">{text}</p>
        </div>
      );
    }
  }

  if (lower === 'keystroke' || lower === 'keystrokes') {
    const text = result.snippet || (result.metadata?.text as string) || '';
    const app = result.metadata?.application as string;
    return (
      <div className="aspect-video bg-[#0F111A] flex flex-col items-start justify-start p-3 gap-1 overflow-hidden">
        <Activity className="w-6 h-6 text-pink-400 shrink-0" />
        {app && <p className="text-xs text-[#9CA3AF] truncate w-full">{app}</p>}
        {text && <p className="text-xs text-[#D1D5DB] line-clamp-3 leading-relaxed font-mono">{text}</p>}
      </div>
    );
  }

  if (lower === 'clipboard') {
    const text = result.snippet || (result.metadata?.text as string) || '';
    return (
      <div className="aspect-video bg-[#0F111A] flex flex-col items-start justify-start p-3 gap-1 overflow-hidden">
        <Clipboard className="w-6 h-6 text-yellow-400 shrink-0" />
        {text && <p className="text-xs text-[#D1D5DB] line-clamp-4 leading-relaxed font-mono">{text}</p>}
      </div>
    );
  }

  if (lower === 'shellhistory' || lower === 'shell_history') {
    const cmd = (result.metadata?.command as string) || '';
    return (
      <div className="aspect-video bg-[#0F111A] flex flex-col items-start justify-start p-3 gap-1 overflow-hidden">
        <Terminal className="w-6 h-6 text-rose-400 shrink-0" />
        {cmd && <p className="text-xs text-[#D1D5DB] line-clamp-2 leading-relaxed font-mono">$ {cmd}</p>}
      </div>
    );
  }

  if (lower === 'browser') {
    const title = (result.metadata?.title as string) || '';
    const url = (result.metadata?.url as string) || '';
    return (
      <div className="aspect-video bg-[#0F111A] flex flex-col items-start justify-start p-3 gap-1 overflow-hidden">
        <Globe className="w-6 h-6 text-cyan-400 shrink-0" />
        {title && <p className="text-xs text-[#F9FAFB] truncate w-full font-medium">{title}</p>}
        {url && <p className="text-xs text-[#9CA3AF] truncate w-full">{url}</p>}
      </div>
    );
  }

  if (lower === 'windowactivity') {
    const app = (result.metadata?.application as string) || '';
    const wt = (result.metadata?.window_title as string) || result.name;
    return (
      <div className="aspect-video bg-[#0F111A] flex flex-col items-start justify-start p-3 gap-1 overflow-hidden">
        <Monitor className="w-6 h-6 text-indigo-400 shrink-0" />
        {app && <p className="text-xs text-[#F9FAFB] truncate w-full font-medium">{app}</p>}
        {wt && <p className="text-xs text-[#9CA3AF] truncate w-full">{wt}</p>}
      </div>
    );
  }

  const iconClass = "w-12 h-12 text-[#4C8BF5]";
  const icon = (() => {
    if (lower === 'mouse') return <Mouse className={iconClass} />;
    if (lower === 'weather') return <Cloud className={iconClass} />;
    if (lower === 'hyprland') return <Monitor className={iconClass} />;
    if (lower === 'processes') return <Cpu className={iconClass} />;
    return <File className={iconClass} />;
  })();

  return (
    <div className="aspect-video bg-[#0F111A] flex items-center justify-center">
      {icon}
    </div>
  );
}

function ModalityDetails({ result }: { result: SearchResult }): JSX.Element {
  const lower = result.modality.toLowerCase();
  const meta = result.metadata ?? {};

  if (lower === 'weather') {
    return (
      <div className="text-sm text-[#9CA3AF] space-y-0.5">
        {meta.temperature != null && <p>🌡️ {String(meta.temperature)}°C</p>}
        {meta.conditions != null && <p>☁️ {String(meta.conditions)}</p>}
        {meta.humidity != null && <p>💧 {String(meta.humidity)}%</p>}
      </div>
    );
  }

  if (lower === 'mouse') {
    return (
      <div className="text-sm text-[#9CA3AF]">
        {meta.x != null && meta.y != null && <p>Position: ({String(meta.x)}, {String(meta.y)})</p>}
        {meta.button != null && <p>Button: {String(meta.button)}</p>}
      </div>
    );
  }

  if (lower === 'processes') {
    return (
      <div className="text-sm text-[#9CA3AF]">
        <p>Process snapshot</p>
      </div>
    );
  }

  if (lower === 'hyprland') {
    return (
      <div className="text-sm text-[#9CA3AF]">
        {meta.workspaces != null && <p>Workspaces: {String(meta.workspaces)}</p>}
      </div>
    );
  }

  if (lower === 'browser') {
    return (
      <div className="text-sm text-[#9CA3AF]">
        {meta.title != null && <p className="truncate" title={String(meta.title)}>{String(meta.title)}</p>}
        {meta.url != null && <p className="truncate text-xs" title={String(meta.url)}>{String(meta.url)}</p>}
      </div>
    );
  }

  if (lower === 'keystroke' || lower === 'keystrokes') {
    return (
      <div className="text-sm text-[#9CA3AF]">
        {meta.application != null && <p className="truncate">{String(meta.application)}</p>}
      </div>
    );
  }

  if (lower === 'shellhistory' || lower === 'shell_history') {
    return (
      <div className="text-sm text-[#9CA3AF]">
        {meta.command != null && <p className="truncate font-mono text-xs">{String(meta.command)}</p>}
      </div>
    );
  }

  if (lower === 'transcription') {
    return (
      <div className="text-sm text-[#9CA3AF] space-y-0.5">
        {meta.model != null && <p className="text-xs">Model: {String(meta.model)}</p>}
        {meta.confidence != null && <p className="text-xs">Confidence: {(Number(meta.confidence) * 100).toFixed(0)}%</p>}
        {meta.text != null && <p className="line-clamp-3 text-xs leading-relaxed">{String(meta.text)}</p>}
      </div>
    );
  }

  return <></>;
}

interface ResultCardProps {
  result?: SearchResult;
  frame?: FrameDataWrapper;
}

function frameToResult(frame: FrameDataWrapper): SearchResult {
  const modalityLower = frame.modality.toLowerCase();
  const preview = frame.image_data_url ?? undefined;
  return {
    id: frame.uuid,
    type: modalityLower === 'audio' || modalityLower === 'microphone' ? 'audio' : 'file',
    name: frame.title ?? frame.text ?? displayNameFromUuid(frame.uuid),
    path: frame.url ?? frame.command ?? frame.working_dir ?? frame.uuid,
    timestamp: (frame.timestamp ?? 0) * 1000,
    source: frame.modality,
    modality: frame.modality,
    preview,
    duration: frame.audio_duration_secs ?? frame.duration_secs ?? undefined,
    metadata: {
      url: frame.url,
      title: frame.title,
      command: frame.command,
      application: frame.application,
      window_title: frame.window_title,
      visit_count: frame.visit_count,
      processes_count: frame.processes?.length ?? 0,
      text: frame.text,
      model: frame.transcription_model,
      confidence: frame.transcription_confidence,
      source_frame_uuid: frame.source_frame_uuid,
      exit_code: frame.exit_code,
      working_dir: frame.working_dir,
    },
  };
}

export default function ResultCard({ result, frame }: ResultCardProps): JSX.Element {
  const effectiveResult = result ?? (frame ? frameToResult(frame) : null);
  if (!effectiveResult) {
    return <></>;
  }
  return (
    <Card className="bg-[#1A1E2E] border-[#232B3D] overflow-hidden">
      <div className="relative">
        <ModalityPreview result={effectiveResult} />
      </div>
      <CardContent className="p-4">
        <div className="flex items-start gap-3">
          <div className="mt-1 shrink-0">
            <ModalityIcon modality={effectiveResult.modality} />
          </div>
          <div className="flex-1 min-w-0">
            <h3 className="font-medium text-[#F9FAFB] truncate" title={effectiveResult.name}>
              <span>{effectiveResult.modality} · </span>
              <span>{effectiveResult.name}</span>
            </h3>
            <div className="text-sm text-[#9CA3AF] mt-1 space-y-0.5">
              <p className="truncate" title={effectiveResult.source}>
                {effectiveResult.source}
              </p>
              <p>{formatDate(effectiveResult.timestamp)}</p>
              {effectiveResult.size != null && <p>{formatFileSize(effectiveResult.size)}</p>}
            </div>
            {effectiveResult.snippet && (
              <p className="text-sm text-[#D1D5DB] mt-2 line-clamp-3">
                {renderHighlightedSnippet(effectiveResult.snippet, effectiveResult.highlightTerms ?? [])}
              </p>
            )}
            <div className="mt-2">
              <ModalityDetails result={effectiveResult} />
            </div>
            {effectiveResult.timestamp > 0 && (
              <button
                type="button"
                onClick={() => {
                  window.dispatchEvent(new CustomEvent('replay-moment', { detail: { timestamp: effectiveResult.timestamp } }));
                  window.dispatchEvent(new CustomEvent('switch-tab', { detail: { tab: 'replay' } }));
                }}
                className="mt-2 flex items-center gap-1 text-xs text-[#4C8BF5] hover:text-[#6BA1F8] transition-colors"
              >
                <Play className="w-3 h-3" />
                Replay this moment
              </button>
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
