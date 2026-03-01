import { useEffect, useMemo, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Cloud,
  HardDrive,
  Laptop,
  Monitor,
  RefreshCw,
  Server,
  Smartphone,
  Wifi,
  WifiOff,
  Zap,
} from 'lucide-react';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { cn } from '../lib/utils';

type DeviceIconType = 'desktop' | 'laptop' | 'phone' | 'cloud';
type ManagedModality = 'screen' | 'camera' | 'microphone' | 'processes' | 'hyprland';

interface CollectorStateWrapper {
  collector_id: string;
  name: string;
  last_seen_secs: number | null;
  total_buffer_size: number;
  upload_lag_bytes: number;
  last_upload_time_secs: number | null;
  source_states: string[];
  source_buffer_sizes: string[];
}

interface BaseConfig {
  enabled: boolean;
}

interface CollectorModel {
  id: string;
  alias: string;
  icon: DeviceIconType;
  online: boolean;
  lastSeenSecs: number | null;
  totalBufferSize: number;
  uploadLagBytes: number;
  lastUploadTimeSecs: number | null;
  sourceStates: string[];
  sourceBufferSizes: string[];
  configs: Partial<Record<ManagedModality, BaseConfig>>;
}

interface NodeOverride {
  alias: string;
  icon: DeviceIconType;
}

interface TopologyOverrides {
  [collectorId: string]: NodeOverride;
}

const MANAGED_MODALITIES: ManagedModality[] = [
  'screen',
  'camera',
  'microphone',
  'processes',
  'hyprland',
];

const MODALITY_LABELS: Record<ManagedModality, string> = {
  screen: 'Screen',
  camera: 'Camera',
  microphone: 'Microphone',
  processes: 'Processes',
  hyprland: 'Window Activity',
};

const MODALITY_PULSE_COLORS: Record<ManagedModality, string> = {
  screen: '#38bdf8',
  camera: '#f97316',
  microphone: '#22c55e',
  processes: '#a855f7',
  hyprland: '#eab308',
};

const ICON_OPTIONS: Array<{ id: DeviceIconType; label: string; icon: typeof Monitor }> = [
  { id: 'desktop', label: 'Desktop', icon: Monitor },
  { id: 'laptop', label: 'Laptop', icon: Laptop },
  { id: 'phone', label: 'Phone', icon: Smartphone },
  { id: 'cloud', label: 'Cloud', icon: Cloud },
];

const OVERRIDE_STORAGE_KEY = 'lifelog-network-topology-overrides';

function readOverrides(): TopologyOverrides {
  try {
    const raw = localStorage.getItem(OVERRIDE_STORAGE_KEY);
    if (!raw) return {};
    const parsed = JSON.parse(raw) as TopologyOverrides;
    return parsed && typeof parsed === 'object' ? parsed : {};
  } catch (_error) {
    return {};
  }
}

function saveOverrides(overrides: TopologyOverrides): void {
  localStorage.setItem(OVERRIDE_STORAGE_KEY, JSON.stringify(overrides));
}

function parseActiveModalities(sourceStates: string[]): ManagedModality[] {
  const detected = new Set<ManagedModality>();
  const joined = sourceStates.join(' ').toLowerCase();
  if (joined.includes('screen')) detected.add('screen');
  if (joined.includes('camera')) detected.add('camera');
  if (joined.includes('microphone') || joined.includes('audio')) detected.add('microphone');
  if (joined.includes('process')) detected.add('processes');
  if (joined.includes('hypr') || joined.includes('window')) detected.add('hyprland');
  return Array.from(detected);
}

function getIcon(icon: DeviceIconType): typeof Monitor {
  return ICON_OPTIONS.find((entry) => entry.id === icon)?.icon ?? Laptop;
}

function formatRelativeMinutes(lastSeenSecs: number | null): string {
  if (!lastSeenSecs) return 'never';
  const minutes = Math.max(0, Math.round((Date.now() / 1000 - lastSeenSecs) / 60));
  return `${minutes}m ago`;
}

function formatClock(lastUploadTimeSecs: number | null): string {
  if (!lastUploadTimeSecs) return 'Never';
  return new Date(lastUploadTimeSecs * 1000).toLocaleTimeString();
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

async function loadCollectorConfigs(collectorId: string): Promise<Partial<Record<ManagedModality, BaseConfig>>> {
  const entries = await Promise.all(
    MANAGED_MODALITIES.map(async (modality) => {
      try {
        const config = await invoke<BaseConfig | null>('get_component_config', {
          collectorId,
          componentType: modality,
        });
        if (!config || typeof config.enabled !== 'boolean') {
          return [modality, undefined] as const;
        }
        return [modality, { enabled: config.enabled }] as const;
      } catch (_error) {
        return [modality, undefined] as const;
      }
    })
  );

  const configs: Partial<Record<ManagedModality, BaseConfig>> = {};
  for (const [modality, config] of entries) {
    if (config) configs[modality] = config;
  }
  return configs;
}

export default function NetworkTopologyDashboard(): JSX.Element {
  const [collectors, setCollectors] = useState<CollectorModel[]>([]);
  const [selectedCollectorId, setSelectedCollectorId] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [actionBusy, setActionBusy] = useState(false);
  const [overrides, setOverrides] = useState<TopologyOverrides>({});

  useEffect(() => {
    setOverrides(readOverrides());
  }, []);

  useEffect(() => {
    loadTopology();
    const timer = setInterval(loadTopology, 20000);
    return () => clearInterval(timer);
  }, []);

  useEffect(() => {
    if (collectors.length === 0) {
      setSelectedCollectorId(null);
      return;
    }
    if (!selectedCollectorId || !collectors.some((collector) => collector.id === selectedCollectorId)) {
      setSelectedCollectorId(collectors[0].id);
    }
  }, [collectors, selectedCollectorId]);

  const selectedCollector = useMemo(
    () => collectors.find((collector) => collector.id === selectedCollectorId) ?? null,
    [collectors, selectedCollectorId]
  );

  async function loadTopology(): Promise<void> {
    setIsLoading(true);
    setError(null);
    try {
      const now = Math.floor(Date.now() / 1000);
      let states: CollectorStateWrapper[] = [];
      try {
        states = await invoke<CollectorStateWrapper[]>('get_system_state');
      } catch (_error) {
        const ids = await invoke<string[]>('get_collector_ids');
        states = ids.map((id) => ({
          collector_id: id,
          name: id,
          last_seen_secs: null,
          total_buffer_size: 0,
          upload_lag_bytes: 0,
          last_upload_time_secs: null,
          source_states: [],
          source_buffer_sizes: [],
        }));
      }

      const configMapEntries = await Promise.all(
        states.map(async (state) => [state.collector_id, await loadCollectorConfigs(state.collector_id)] as const)
      );
      const configMap = new Map(configMapEntries);

      const nextCollectors: CollectorModel[] = states.map((state) => {
        const override = overrides[state.collector_id];
        return {
          id: state.collector_id,
          alias: override?.alias || state.name || state.collector_id,
          icon: override?.icon || 'laptop',
          online: state.last_seen_secs !== null && (now - state.last_seen_secs) < 120,
          lastSeenSecs: state.last_seen_secs,
          totalBufferSize: state.total_buffer_size,
          uploadLagBytes: state.upload_lag_bytes,
          lastUploadTimeSecs: state.last_upload_time_secs,
          sourceStates: state.source_states,
          sourceBufferSizes: state.source_buffer_sizes,
          configs: configMap.get(state.collector_id) ?? {},
        };
      });

      setCollectors(nextCollectors);
    } catch (err) {
      console.error('[NetworkTopology] Failed to load topology', err);
      setError('Failed to load topology data from backend.');
    } finally {
      setIsLoading(false);
    }
  }

  async function setModalityEnabled(collectorId: string, modality: ManagedModality, enabled: boolean): Promise<void> {
    const collector = collectors.find((entry) => entry.id === collectorId);
    const existingConfig = collector?.configs[modality];
    if (!existingConfig) {
      setNotice(`Cannot update ${MODALITY_LABELS[modality]} on ${collectorId}: missing backend config.`);
      return;
    }

    setActionBusy(true);
    setNotice(null);
    try {
      await invoke('set_component_config', {
        collectorId,
        componentType: modality,
        configValue: { ...existingConfig, enabled },
      });
      await loadTopology();
      setNotice(`${MODALITY_LABELS[modality]} ${enabled ? 'enabled' : 'disabled'} for ${collector.alias}.`);
    } catch (err) {
      console.error('[NetworkTopology] Failed to update modality', err);
      setNotice(`Failed to update ${MODALITY_LABELS[modality]} for ${collector?.alias ?? collectorId}.`);
    } finally {
      setActionBusy(false);
    }
  }

  async function setPaused(collectorId: string, paused: boolean): Promise<void> {
    const collector = collectors.find((entry) => entry.id === collectorId);
    if (!collector) return;

    setActionBusy(true);
    setNotice(null);
    try {
      const updates = MANAGED_MODALITIES
        .map((modality) => ({ modality, config: collector.configs[modality] }))
        .filter((entry) => !!entry.config) as Array<{ modality: ManagedModality; config: BaseConfig }>;

      if (updates.length === 0) {
        throw new Error('No mutable modalities available for this collector.');
      }

      await Promise.all(
        updates.map((entry) =>
          invoke('set_component_config', {
            collectorId,
            componentType: entry.modality,
            configValue: { ...entry.config, enabled: !paused },
          })
        )
      );

      await loadTopology();
      setNotice(`${paused ? 'Paused' : 'Resumed'} capture for ${collector.alias}.`);
    } catch (err) {
      console.error('[NetworkTopology] Pause/resume failed', err);
      setNotice(`Failed to ${paused ? 'pause' : 'resume'} ${collector.alias}.`);
    } finally {
      setActionBusy(false);
    }
  }

  async function handleForceSync(collectorId: string): Promise<void> {
    setActionBusy(true);
    setNotice(null);
    try {
      await invoke('force_collector_sync', { collectorId });
      setNotice('Force sync command sent.');
    } catch (err) {
      console.error('[NetworkTopology] Force sync unavailable', err);
      setNotice('Force sync RPC is not exposed by the current backend API.');
    } finally {
      setActionBusy(false);
    }
  }

  function saveSelectedOverride(alias: string, icon: DeviceIconType): void {
    if (!selectedCollector) return;
    const next = {
      ...overrides,
      [selectedCollector.id]: {
        alias: alias.trim() || selectedCollector.id,
        icon,
      },
    };
    setOverrides(next);
    saveOverrides(next);
    setCollectors((previous) =>
      previous.map((collector) =>
        collector.id === selectedCollector.id
          ? { ...collector, alias: next[selectedCollector.id].alias, icon: next[selectedCollector.id].icon }
          : collector
      )
    );
    setNotice('Saved local alias/icon override for this topology view.');
  }

  const graphLayout = useMemo(() => {
    const count = collectors.length;
    if (count === 0) return [] as Array<{ id: string; x: number; y: number }>;

    const startDeg = 215;
    const endDeg = 325;
    const span = count === 1 ? 0 : (endDeg - startDeg) / (count - 1);
    return collectors.map((collector, index) => {
      const angle = (startDeg + span * index) * (Math.PI / 180);
      const x = 500 + Math.cos(angle) * 360;
      const y = 360 + Math.sin(angle) * 230;
      return { id: collector.id, x, y };
    });
  }, [collectors]);

  return (
    <div className="p-6 md:p-8 h-full flex flex-col gap-5">
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div>
          <h1 className="title">Network</h1>
          <p className="subtitle mt-1">Interactive topology for server and collectors.</p>
        </div>
        <Button onClick={loadTopology} variant="secondary" disabled={isLoading}>
          <RefreshCw className={cn('w-4 h-4 mr-2', isLoading && 'animate-spin')} />
          Refresh
        </Button>
      </div>

      {error && (
        <div className="rounded-lg border border-red-500/40 bg-red-500/10 text-red-300 px-4 py-3 text-sm">
          {error}
        </div>
      )}

      {notice && (
        <div className="rounded-lg border border-[#2A3142] bg-[#1A1E2E] text-[#CBD5E1] px-4 py-3 text-sm">
          {notice}
        </div>
      )}

      <div className="rounded-xl border border-[#2A3142] bg-[#151A29] p-4 md:p-6 relative overflow-hidden min-h-[420px]">
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_center,_rgba(76,139,245,0.16)_0%,_rgba(15,17,26,0)_60%)] pointer-events-none" />
        <svg viewBox="0 0 1000 640" className="w-full h-[420px] md:h-[520px] relative z-10">
          <defs>
            <filter id="lineGlow" x="-200%" y="-200%" width="400%" height="400%">
              <feGaussianBlur stdDeviation="2.8" result="blur" />
              <feMerge>
                <feMergeNode in="blur" />
                <feMergeNode in="SourceGraphic" />
              </feMerge>
            </filter>
          </defs>

          <g>
            {graphLayout.map((position) => {
              const collector = collectors.find((entry) => entry.id === position.id);
              if (!collector) return null;
              const lineColor = collector.online ? '#4C8BF5' : '#4B5563';
              const activeModalities = parseActiveModalities(collector.sourceStates);

              return (
                <g key={`edge-${collector.id}`}>
                  <line
                    x1={500}
                    y1={120}
                    x2={position.x}
                    y2={position.y}
                    stroke={lineColor}
                    strokeWidth={collector.online ? 2.6 : 1.6}
                    strokeOpacity={collector.online ? 0.85 : 0.45}
                    filter="url(#lineGlow)"
                  />
                  {activeModalities.map((modality, index) => (
                    <circle key={`${collector.id}-${modality}`} r="4" fill={MODALITY_PULSE_COLORS[modality]}>
                      <animateMotion
                        dur={`${1.8 + index * 0.4}s`}
                        repeatCount="indefinite"
                        path={`M500,120 L${position.x},${position.y}`}
                      />
                    </circle>
                  ))}
                </g>
              );
            })}
          </g>
        </svg>

        <div className="absolute left-1/2 top-8 -translate-x-1/2 z-20">
          <button
            type="button"
            className={cn(
              'min-w-[180px] rounded-xl border px-4 py-3 backdrop-blur-sm text-left transition',
              'bg-[#1A1E2E]/95 border-[#4C8BF5]/50 shadow-[0_0_35px_rgba(76,139,245,0.28)]'
            )}
          >
            <div className="flex items-center gap-2 text-[#F9FAFB]">
              <Server className="w-4 h-4 text-[#4C8BF5]" />
              <span className="font-semibold text-sm">Lifelog Server</span>
            </div>
            <p className="text-xs text-[#9CA3AF] mt-1">Control plane + data plane authority</p>
          </button>
        </div>

        {graphLayout.map((position) => {
          const collector = collectors.find((entry) => entry.id === position.id);
          if (!collector) return null;
          const NodeIcon = getIcon(collector.icon);
          const activeModalities = parseActiveModalities(collector.sourceStates);

          return (
            <div
              key={`node-${collector.id}`}
              className="absolute z-20"
              style={{ left: `${(position.x / 1000) * 100}%`, top: `${(position.y / 640) * 100}%`, transform: 'translate(-50%, -50%)' }}
            >
              <button
                type="button"
                data-testid={`collector-node-${collector.id}`}
                onClick={() => setSelectedCollectorId(collector.id)}
                className={cn(
                  'w-[190px] rounded-xl border px-3 py-2 text-left backdrop-blur-sm transition',
                  selectedCollectorId === collector.id
                    ? 'border-[#4C8BF5] bg-[#1F2638]/95 shadow-[0_0_26px_rgba(76,139,245,0.25)]'
                    : 'border-[#2A3142] bg-[#171D2D]/90 hover:border-[#3A4A66]'
                )}
              >
                <div className="flex items-center justify-between gap-2">
                  <div className="flex items-center gap-2 min-w-0">
                    <NodeIcon className="w-4 h-4 text-[#A5B4FC]" />
                    <span className="text-sm font-semibold text-[#F9FAFB] truncate">{collector.alias}</span>
                  </div>
                  <span className={cn('w-2.5 h-2.5 rounded-full', collector.online ? 'bg-green-400' : 'bg-slate-500')} />
                </div>
                <p className="text-[11px] text-[#94A3B8] truncate mt-1">{collector.id}</p>
                <div className="flex items-center gap-1 mt-2 flex-wrap">
                  {activeModalities.length === 0 && (
                    <span className="text-[10px] px-1.5 py-0.5 rounded bg-[#0F111A] text-[#94A3B8]">idle</span>
                  )}
                  {activeModalities.slice(0, 3).map((modality) => (
                    <span
                      key={`${collector.id}-${modality}`}
                      className="text-[10px] px-1.5 py-0.5 rounded"
                      style={{ backgroundColor: `${MODALITY_PULSE_COLORS[modality]}26`, color: MODALITY_PULSE_COLORS[modality] }}
                    >
                      {MODALITY_LABELS[modality]}
                    </span>
                  ))}
                </div>
              </button>
            </div>
          );
        })}
      </div>

      {collectors.length === 0 && !isLoading && (
        <div className="rounded-lg border border-[#2A3142] bg-[#1A1E2E] p-4 text-sm text-[#94A3B8]">
          No collectors found.
        </div>
      )}

      {selectedCollector && (
        <CollectorControlPanel
          collector={selectedCollector}
          busy={actionBusy}
          onToggleModality={setModalityEnabled}
          onPause={setPaused}
          onForceSync={handleForceSync}
          onSaveOverride={saveSelectedOverride}
        />
      )}
    </div>
  );
}

interface CollectorControlPanelProps {
  collector: CollectorModel;
  busy: boolean;
  onToggleModality: (collectorId: string, modality: ManagedModality, enabled: boolean) => Promise<void>;
  onPause: (collectorId: string, paused: boolean) => Promise<void>;
  onForceSync: (collectorId: string) => Promise<void>;
  onSaveOverride: (alias: string, icon: DeviceIconType) => void;
}

function CollectorControlPanel({
  collector,
  busy,
  onToggleModality,
  onPause,
  onForceSync,
  onSaveOverride,
}: CollectorControlPanelProps): JSX.Element {
  const [aliasInput, setAliasInput] = useState(collector.alias);
  const [iconInput, setIconInput] = useState<DeviceIconType>(collector.icon);

  useEffect(() => {
    setAliasInput(collector.alias);
    setIconInput(collector.icon);
  }, [collector.alias, collector.icon, collector.id]);

  const allKnownConfigs = MANAGED_MODALITIES.filter((modality) => !!collector.configs[modality]);
  const isPaused = allKnownConfigs.length > 0 && allKnownConfigs.every((modality) => !collector.configs[modality]?.enabled);

  return (
    <div className="rounded-xl border border-[#2A3142] bg-[#1A1E2E] p-5 grid grid-cols-1 lg:grid-cols-3 gap-5">
      <div className="space-y-4">
        <div>
          <p className="text-xs uppercase tracking-wide text-[#94A3B8]">Collector</p>
          <h2 className="text-lg font-semibold text-[#F9FAFB]">{collector.alias}</h2>
          <p className="text-xs text-[#94A3B8] mt-1">{collector.id}</p>
        </div>

        <div className="grid grid-cols-2 gap-3 text-sm">
          <StatChip
            label="Status"
            value={collector.online ? 'Online' : 'Offline'}
            icon={collector.online ? Wifi : WifiOff}
            valueClass={collector.online ? 'text-green-300' : 'text-slate-300'}
          />
          <StatChip label="Seen" value={formatRelativeMinutes(collector.lastSeenSecs)} icon={RefreshCw} />
          <StatChip label="Backlog" value={formatBytes(collector.uploadLagBytes)} icon={HardDrive} />
          <StatChip label="Buffer" value={`${collector.totalBufferSize}`} icon={Zap} />
        </div>

        <p className="text-xs text-[#94A3B8]">Last upload: {formatClock(collector.lastUploadTimeSecs)}</p>
      </div>

      <div>
        <p className="text-xs uppercase tracking-wide text-[#94A3B8] mb-3">Capture Controls</p>
        <div className="space-y-2">
          {MANAGED_MODALITIES.map((modality) => {
            const config = collector.configs[modality];
            return (
              <div
                key={`${collector.id}-${modality}`}
                className="flex items-center justify-between rounded-lg border border-[#2A3142] bg-[#151A29] px-3 py-2"
              >
                <div>
                  <p className="text-sm text-[#F9FAFB]">{MODALITY_LABELS[modality]}</p>
                  <p className="text-[11px] text-[#94A3B8]">{config ? (config.enabled ? 'Enabled' : 'Disabled') : 'Unavailable'}</p>
                </div>
                <Button
                  size="sm"
                  variant="secondary"
                  disabled={busy || !config}
                  onClick={() => onToggleModality(collector.id, modality, !config?.enabled)}
                >
                  {config?.enabled ? 'Disable' : 'Enable'}
                </Button>
              </div>
            );
          })}
        </div>

        <div className="flex flex-wrap gap-2 mt-4">
          <Button size="sm" variant="secondary" disabled={busy} onClick={() => onPause(collector.id, !isPaused)}>
            {isPaused ? 'Resume Capture' : 'Pause Capture'}
          </Button>
          <Button size="sm" variant="secondary" disabled={busy} onClick={() => onForceSync(collector.id)}>
            Force Sync
          </Button>
        </div>
      </div>

      <div>
        <p className="text-xs uppercase tracking-wide text-[#94A3B8] mb-3">Node Appearance</p>
        <label className="text-xs text-[#94A3B8]">Alias</label>
        <Input
          className="mt-1 bg-[#151A29] border-[#2A3142] text-[#F9FAFB]"
          value={aliasInput}
          onChange={(event) => setAliasInput(event.target.value)}
          placeholder="Collector alias"
        />

        <label className="text-xs text-[#94A3B8] mt-3 block">Icon</label>
        <div className="grid grid-cols-2 gap-2 mt-1">
          {ICON_OPTIONS.map((option) => {
            const Icon = option.icon;
            return (
              <button
                key={option.id}
                type="button"
                onClick={() => setIconInput(option.id)}
                className={cn(
                  'flex items-center gap-2 rounded-lg border px-3 py-2 text-sm',
                  iconInput === option.id
                    ? 'border-[#4C8BF5] bg-[#4C8BF5]/10 text-[#BFDBFE]'
                    : 'border-[#2A3142] bg-[#151A29] text-[#CBD5E1] hover:border-[#3A4A66]'
                )}
              >
                <Icon className="w-4 h-4" />
                <span>{option.label}</span>
              </button>
            );
          })}
        </div>

        <Button size="sm" className="mt-4" onClick={() => onSaveOverride(aliasInput, iconInput)}>
          Save Alias/Icon
        </Button>
        <p className="text-[11px] text-[#94A3B8] mt-2">Saved locally in interface storage.</p>
      </div>
    </div>
  );
}

interface StatChipProps {
  label: string;
  value: string;
  icon: typeof HardDrive;
  valueClass?: string;
}

function StatChip({ label, value, icon: Icon, valueClass }: StatChipProps): JSX.Element {
  return (
    <div className="rounded-lg border border-[#2A3142] bg-[#151A29] px-3 py-2">
      <p className="text-[11px] text-[#94A3B8] flex items-center gap-1">
        <Icon className="w-3 h-3" />
        {label}
      </p>
      <p className={cn('text-sm text-[#F9FAFB] mt-1 truncate', valueClass)}>{value}</p>
    </div>
  );
}
