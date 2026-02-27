import { Image, FileAudio, File, Mouse, Cloud, Monitor, Cpu, Clipboard, Globe, Terminal, Activity } from 'lucide-react';
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
}

function formatDate(timestamp: number): string {
  return new Date(timestamp).toLocaleString();
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
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
  if (lower === 'keystrokes') return <Activity className="w-5 h-5 text-pink-400" />;
  return <File className="w-5 h-5 text-gray-400" />;
}

function ModalityPreview({ result }: { result: SearchResult }): JSX.Element | null {
  const lower = result.modality.toLowerCase();

  if ((lower === 'screen' || lower === 'camera') && result.preview) {
    return (
      <div className="aspect-video bg-[#0F111A] flex items-center justify-center overflow-hidden">
        <img src={result.preview} alt={result.name} className="w-full h-full object-cover" />
      </div>
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

  const iconClass = "w-12 h-12 text-[#4C8BF5]";
  const icon = (() => {
    if (lower === 'mouse') return <Mouse className={iconClass} />;
    if (lower === 'weather') return <Cloud className={iconClass} />;
    if (lower === 'hyprland' || lower === 'windowactivity') return <Monitor className={iconClass} />;
    if (lower === 'processes') return <Cpu className={iconClass} />;
    if (lower === 'clipboard') return <Clipboard className={iconClass} />;
    if (lower === 'browser') return <Globe className={iconClass} />;
    if (lower === 'shellhistory' || lower === 'shell_history') return <Terminal className={iconClass} />;
    if (lower === 'keystrokes') return <Activity className={iconClass} />;
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

  if (lower === 'keystrokes') {
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

  return <></>;
}

interface ResultCardProps {
  result: SearchResult;
}

export default function ResultCard({ result }: ResultCardProps): JSX.Element {
  return (
    <Card className="bg-[#1A1E2E] border-[#232B3D] overflow-hidden">
      <div className="relative">
        <ModalityPreview result={result} />
      </div>
      <CardContent className="p-4">
        <div className="flex items-start gap-3">
          <div className="mt-1 shrink-0">
            <ModalityIcon modality={result.modality} />
          </div>
          <div className="flex-1 min-w-0">
            <h3 className="font-medium text-[#F9FAFB] truncate" title={result.name}>
              {result.modality} · {result.name}
            </h3>
            <div className="text-sm text-[#9CA3AF] mt-1 space-y-0.5">
              <p className="truncate" title={result.source}>
                {result.source}
              </p>
              <p>{formatDate(result.timestamp)}</p>
              {result.size != null && <p>{formatFileSize(result.size)}</p>}
            </div>
            <div className="mt-2">
              <ModalityDetails result={result} />
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
