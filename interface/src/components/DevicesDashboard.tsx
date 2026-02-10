import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { 
  Laptop, 
  Camera, 
  Mic, 
  Monitor, 
  RefreshCw,
  AlertCircle
} from 'lucide-react';
import { Button } from './ui/button';
import { Switch } from './ui/switch';
import { Card, CardContent, CardHeader, CardTitle } from './ui/card';
import ScreenDashboard from './ScreenDashboard';
import { cn } from '../lib/utils';

// Configuration Interfaces
interface BaseConfig {
  enabled: boolean;
  interval?: number;
  output_dir?: string;
}

interface CameraConfig extends BaseConfig {
  device?: string;
  fps?: number;
  resolution_x?: number;
  resolution_y?: number;
  timestamp_format?: string;
}

interface MicrophoneConfig extends BaseConfig {
  chunk_duration_secs?: number;
  capture_interval_secs?: number;
  sample_rate?: number;
  bits_per_sample?: number;
  channels?: number;
  timestamp_format?: string;
}

interface CollectorStatus {
  id: string;
  isOnline: boolean; // Placeholder for now
  lastSeen?: Date;
}

export default function DevicesDashboard(): JSX.Element {
  const [collectors, setCollectors] = useState<CollectorStatus[]>([]);
  const [selectedCollectorId, setSelectedCollectorId] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Config States for the selected collector
  const [cameraConfig, setCameraConfig] = useState<CameraConfig | null>(null);
  const [micConfig, setMicConfig] = useState<MicrophoneConfig | null>(null);
  const [loadingConfigs, setLoadingConfigs] = useState(false);

  useEffect(() => {
    loadCollectors();
  }, []);

  useEffect(() => {
    if (selectedCollectorId) {
      loadDeviceConfigs(selectedCollectorId);
    }
  }, [selectedCollectorId]);

  async function loadCollectors() {
    setIsLoading(true);
    setError(null);
    try {
      const ids = await invoke<string[]>('get_collector_ids');
      const collectorObjects = ids.map(id => ({
        id,
        isOnline: true, // Mock status for now as backend doesn't provide heartbeat yet
        lastSeen: new Date()
      }));
      setCollectors(collectorObjects);
      
      // Auto-select first device if none selected
      if (collectorObjects.length > 0 && !selectedCollectorId) {
        setSelectedCollectorId(collectorObjects[0].id);
      }
    } catch (err) {
      console.error('Failed to load collectors:', err);
      setError('Failed to load devices. Is the server running?');
    } finally {
      setIsLoading(false);
    }
  }

  async function loadDeviceConfigs(collectorId: string) {
    setLoadingConfigs(true);
    try {
      // Load Camera Config
      try {
        const cam = await invoke<CameraConfig>('get_component_config', {
          collectorId,
          componentType: 'camera'
        });
        setCameraConfig(cam);
      } catch (e) {
        console.warn('Failed to load camera config:', e);
        setCameraConfig(null);
      }

      // Load Mic Config
      try {
        const mic = await invoke<MicrophoneConfig>('get_component_config', {
          collectorId,
          componentType: 'microphone'
        });
        setMicConfig(mic);
      } catch (e) {
        console.warn('Failed to load mic config:', e);
        setMicConfig(null);
      }

    } finally {
      setLoadingConfigs(false);
    }
  }

  async function toggleComponent(type: 'camera' | 'microphone', currentConfig: BaseConfig) {
    if (!selectedCollectorId) return;
    
    const newEnabled = !currentConfig.enabled;
    const newConfig = { ...currentConfig, enabled: newEnabled };

    try {
      await invoke('set_component_config', {
        collectorId: selectedCollectorId,
        componentType: type,
        configValue: newConfig
      });
      
      // Update local state
      if (type === 'camera') setCameraConfig(newConfig as CameraConfig);
      if (type === 'microphone') setMicConfig(newConfig as MicrophoneConfig);
      
    } catch (err) {
      console.error(`Failed to toggle ${type}:`, err);
      alert(`Failed to update ${type} settings.`);
    }
  }

  return (
    <div className="p-6 md:p-8 space-y-6 h-full flex flex-col">
      <div className="flex items-center justify-between mb-2">
        <div>
          <div className="flex items-center gap-3 mb-2">
            <Laptop className="w-8 h-8 text-[#4C8BF5]" />
            <h1 className="title">Devices</h1>
          </div>
          <p className="subtitle">Manage connected collectors and their capture settings</p>
        </div>
        <Button onClick={loadCollectors} variant="secondary" disabled={isLoading}>
          <RefreshCw className={cn("w-4 h-4 mr-2", isLoading && "animate-spin")} />
          Refresh
        </Button>
      </div>

      {error && (
        <div className="bg-red-500/10 border border-red-500/50 text-red-500 p-4 rounded-lg flex items-center gap-3">
          <AlertCircle className="w-5 h-5" />
          <p>{error}</p>
        </div>
      )}

      <div className="grid grid-cols-1 lg:grid-cols-4 gap-6 flex-1 overflow-hidden">
        {/* Device List Sidebar */}
        <div className="lg:col-span-1 space-y-4 overflow-y-auto pr-2">
          {collectors.length === 0 && !isLoading && (
            <div className="text-center p-8 bg-[#1A1E2E] rounded-lg border border-[#232B3D]">
              <p className="text-[#9CA3AF]">No devices found</p>
            </div>
          )}
          
          {collectors.map(collector => (
            <div 
              key={collector.id}
              onClick={() => setSelectedCollectorId(collector.id)}
              className={cn(
                "p-4 rounded-lg border cursor-pointer transition-all hover:bg-[#232B3D]",
                selectedCollectorId === collector.id 
                  ? "bg-[#232B3D] border-[#4C8BF5] shadow-[0_0_10px_rgba(76,139,245,0.1)]" 
                  : "bg-[#1A1E2E] border-[#232B3D]"
              )}
            >
              <div className="flex items-center gap-3 mb-2">
                <Laptop className={cn(
                  "w-5 h-5",
                  selectedCollectorId === collector.id ? "text-[#4C8BF5]" : "text-[#9CA3AF]"
                )} />
                <span className="font-medium text-[#F9FAFB] truncate">{collector.id}</span>
              </div>
              <div className="flex items-center justify-between text-xs">
                <span className={cn(
                  "flex items-center gap-1.5",
                  collector.isOnline ? "text-green-400" : "text-[#9CA3AF]"
                )}>
                  <span className={cn(
                    "w-2 h-2 rounded-full",
                    collector.isOnline ? "bg-green-400" : "bg-[#9CA3AF]"
                  )} />
                  {collector.isOnline ? "Online" : "Offline"}
                </span>
                <span className="text-[#9CA3AF]">
                  Last seen: Today
                </span>
              </div>
            </div>
          ))}
        </div>

        {/* Device Details Area */}
        <div className="lg:col-span-3 flex flex-col gap-6 overflow-y-auto pb-8">
          {selectedCollectorId ? (
            <>
              {/* Quick Toggles */}
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {/* Camera Toggle */}
                <Card className="bg-[#1A1E2E] border-[#232B3D]">
                  <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                    <CardTitle className="text-sm font-medium text-[#F9FAFB]">Camera</CardTitle>
                    <Camera className="h-4 w-4 text-[#9CA3AF]" />
                  </CardHeader>
                  <CardContent>
                    {cameraConfig ? (
                      <div className="flex items-center justify-between mt-2">
                        <div className="text-2xl font-bold text-[#F9FAFB]">
                          {cameraConfig.enabled ? 'On' : 'Off'}
                        </div>
                        <Switch 
                          checked={cameraConfig.enabled}
                          onCheckedChange={() => toggleComponent('camera', cameraConfig)}
                          disabled={loadingConfigs}
                        />
                      </div>
                    ) : (
                      <div className="text-sm text-[#9CA3AF] mt-2">
                        {loadingConfigs ? 'Loading...' : 'Not Configured'}
                      </div>
                    )}
                    {cameraConfig && (
                      <p className="text-xs text-[#9CA3AF] mt-1">
                        {cameraConfig.fps} FPS • {cameraConfig.interval}s Interval
                      </p>
                    )}
                  </CardContent>
                </Card>

                {/* Microphone Toggle */}
                <Card className="bg-[#1A1E2E] border-[#232B3D]">
                  <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                    <CardTitle className="text-sm font-medium text-[#F9FAFB]">Microphone</CardTitle>
                    <Mic className="h-4 w-4 text-[#9CA3AF]" />
                  </CardHeader>
                  <CardContent>
                    {micConfig ? (
                      <div className="flex items-center justify-between mt-2">
                        <div className="text-2xl font-bold text-[#F9FAFB]">
                          {micConfig.enabled ? 'On' : 'Off'}
                        </div>
                        <Switch 
                          checked={micConfig.enabled}
                          onCheckedChange={() => toggleComponent('microphone', micConfig)}
                          disabled={loadingConfigs}
                        />
                      </div>
                    ) : (
                      <div className="text-sm text-[#9CA3AF] mt-2">
                        {loadingConfigs ? 'Loading...' : 'Not Configured'}
                      </div>
                    )}
                     {micConfig && (
                      <p className="text-xs text-[#9CA3AF] mt-1">
                        {micConfig.chunk_duration_secs}s Chunks • {micConfig.capture_interval_secs}s Interval
                      </p>
                    )}
                  </CardContent>
                </Card>
              </div>

              {/* Detailed Screen Config (Reusing ScreenDashboard) */}
              <div className="bg-[#1A1E2E] rounded-lg border border-[#232B3D] overflow-hidden">
                 <div className="p-4 border-b border-[#232B3D] flex items-center gap-2">
                    <Monitor className="w-5 h-5 text-[#4C8BF5]" />
                    <h2 className="font-medium text-[#F9FAFB]">Screen Recording</h2>
                 </div>
                 <div className="p-0">
                    <ScreenDashboard collectorId={selectedCollectorId} />
                 </div>
              </div>

            </>
          ) : (
             <div className="flex items-center justify-center h-full text-[#9CA3AF]">
                Select a device to view settings
             </div>
          )}
        </div>
      </div>
    </div>
  );
}
