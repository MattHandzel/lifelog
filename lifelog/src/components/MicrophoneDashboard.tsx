import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button } from './ui/button';
import { Slider } from './ui/slider';
import { Switch } from './ui/switch';
import { Mic, Clock, Square, Settings, Play, Pause, ExternalLink } from 'lucide-react';
import { cn } from '../lib/utils';

interface MicrophoneSettings {
  enabled: boolean; // Auto-recording enabled
  chunk_duration_secs: number;
  output_dir: string;
  channels: number;
  sample_rate: number;
  bits_per_sample: number;
  timestamp_format: string;
  capture_interval_secs: number; // How often to auto-record in seconds
}

interface AudioFile {
  path: string;
  filename: string;
  duration: number;
  created_at: string;
  size: number;
}

interface RecordingStatus {
  is_recording: boolean;
  is_paused: boolean;
  auto_recording_enabled: boolean;
}

const MicrophoneDashboard: React.FC = () => {
  // Dashboard states
  const [isRecording, setIsRecording] = useState<boolean>(false);
  const [isPaused, setIsPaused] = useState<boolean>(false);
  const [recordings, setRecordings] = useState<AudioFile[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  // Settings state from the backend
  const [settings, setSettings] = useState<MicrophoneSettings>({
    enabled: false,
    chunk_duration_secs: 60,
    output_dir: '',
    channels: 1,
    sample_rate: 44100,
    bits_per_sample: 16,
    timestamp_format: '%Y-%m-%d_%H-%M-%S',
    capture_interval_secs: 300, // Default 5 minutes
  });

  // Temporary settings for the settings panel (card)
  const [tempSettings, setTempSettings] = useState<{
    autoCapture: boolean;
    recordingDuration: number;
    captureInterval: number;
  }>({
    autoCapture: false,
    recordingDuration: 60,
    captureInterval: 300,
  });
  const [showSettings, setShowSettings] = useState<boolean>(false);
  const [isSavingSettings, setIsSavingSettings] = useState<boolean>(false);

  const statusCheckInterval = useRef<number | null>(null);

  useEffect(() => {
    loadSettings();
    fetchRecordings();
    checkRecordingStatus();
    
    // Start periodic status check
    statusCheckInterval.current = window.setInterval(checkRecordingStatus, 2000);
    
    return () => {
      if (statusCheckInterval.current) {
        clearInterval(statusCheckInterval.current);
      }
    };
  }, []);

  // Ensure recording duration is less than capture interval (80% max)
  useEffect(() => {
    if (tempSettings.recordingDuration >= tempSettings.captureInterval) {
      const newDuration = Math.floor(tempSettings.captureInterval * 0.8);
      setTempSettings({ ...tempSettings, recordingDuration: newDuration });
    }
  }, [tempSettings.captureInterval]);

  const loadSettings = async () => {
    try {
      setErrorMessage(null);
      const config = await invoke<MicrophoneSettings>('get_microphone_config');
      console.log('Loaded microphone config:', config);
      setSettings(config);
      setTempSettings({
        autoCapture: config.enabled,
        recordingDuration: config.chunk_duration_secs,
        captureInterval: config.capture_interval_secs || 300,
      });
    } catch (error) {
      console.error('Failed to load microphone settings:', error);
      setErrorMessage(`Failed to load settings: ${error}`);
    }
  };

  const saveSettings = async () => {
    try {
      setIsSavingSettings(true);
      setErrorMessage(null);
      const updatedSettings = {
        ...settings,
        enabled: tempSettings.autoCapture,
        chunk_duration_secs: tempSettings.recordingDuration,
        capture_interval_secs: tempSettings.captureInterval,
      };

      // Update configuration on the backend
      await invoke('update_microphone_config', { config: updatedSettings });
      
      setSettings(updatedSettings);
      
      // No need to call both update_microphone_config and update_microphone_settings
      // as update_microphone_config now handles runtime updates as well
      
      setShowSettings(false);
    } catch (error) {
      console.error('Failed to save microphone settings:', error);
      setErrorMessage(`Failed to save settings: ${error}`);
    } finally {
      setIsSavingSettings(false);
    }
  };

  const fetchRecordings = async () => {
    try {
      setIsLoading(true);
      setErrorMessage(null);
      console.log('Fetching audio recordings...');
      const files = await invoke<AudioFile[]>('get_audio_files', {
        page: 1,
        pageSize: 20,
      });
      console.log('Retrieved audio files:', files);
      setRecordings(Array.isArray(files) ? files : []);
    } catch (error) {
      console.error('Failed to fetch recordings:', error);
      setErrorMessage(`Failed to fetch recordings: ${error}`);
      setRecordings([]);
    } finally {
      setIsLoading(false);
    }
  };

  const handleStartRecording = async () => {
    try {
      setErrorMessage(null);
      await invoke('start_recording');
      setIsRecording(true);
      setIsPaused(false);
    } catch (error) {
      console.error('Failed to start recording:', error);
      setErrorMessage(`Failed to start recording: ${error}`);
    }
  };

  const handlePauseRecording = async () => {
    try {
      setErrorMessage(null);
      if (isPaused) {
        await invoke('resume_recording');
        setIsPaused(false);
      } else {
        await invoke('pause_recording');
        setIsPaused(true);
      }
    } catch (error) {
      console.error('Failed to pause/resume recording:', error);
      setErrorMessage(`Failed to pause/resume recording: ${error}`);
    }
  };

  const handleStopRecording = async () => {
    try {
      setErrorMessage(null);
      await invoke('stop_recording');
      setIsRecording(false);
      setIsPaused(false);
      fetchRecordings();
    } catch (error) {
      console.error('Failed to stop recording:', error);
      setErrorMessage(`Failed to stop recording: ${error}`);
    }
  };

  const handleOpenTerminalForRecording = async () => {
    try {
      setErrorMessage(null);
      await invoke('open_terminal_for_recording');
    } catch (error) {
      console.error('Failed to open terminal for recording:', error);
      setErrorMessage(`Failed to open terminal: ${error}`);
    }
  };

  const checkRecordingStatus = async () => {
    try {
      const status = await invoke<RecordingStatus>('get_recording_status');
      setIsRecording(status.is_recording);
      setIsPaused(status.is_paused);
      
      // Also update the settings if auto recording state changed
      if (settings.enabled !== status.auto_recording_enabled) {
        setSettings({
          ...settings,
          enabled: status.auto_recording_enabled
        });
        
        setTempSettings({
          ...tempSettings,
          autoCapture: status.auto_recording_enabled
        });
      }
    } catch (error) {
      console.error('Failed to check recording status:', error);
    }
  };

  // Helper to format seconds into a friendly string (e.g., "5m 30s")
  const formatTimeForDisplay = (seconds: number): string => {
    if (seconds < 60) {
      return `${seconds}s`;
    } else {
      const minutes = Math.floor(seconds / 60);
      const remainingSecs = seconds % 60;
      return remainingSecs === 0 ? `${minutes}m` : `${minutes}m ${remainingSecs}s`;
    }
  };

  // Calculate maximum recording duration as 80% of the capture interval
  const getMaxRecordingDuration = () => Math.floor(tempSettings.captureInterval * 0.8);

  return (
    <div className="p-6 md:p-8 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 flex items-center justify-center bg-blue-900/20 rounded-lg">
            <Mic className="h-5 w-5 text-blue-400" />
          </div>
          <div>
            <h1 className="text-xl font-semibold text-white">Audio Capture</h1>
            <p className="text-sm text-gray-400">Manage audio recording settings</p>
          </div>
        </div>
        <Button onClick={() => setShowSettings(!showSettings)} className="btn-secondary flex items-center gap-2">
          <Settings className="h-4 w-4" />
          <span>Settings</span>
        </Button>
      </div>

      {/* Microphone Settings Panel (Card) */}
      {showSettings && (
        <div className="card mb-8 bg-[#151926] rounded-lg shadow-2xl">
          <div className="p-6">
            <h2 className="text-lg font-medium text-[#F9FAFB] mb-6">Microphone Settings</h2>
            <div className="space-y-6">
              {/* Auto Recording Toggle */}
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <div className="p-2 bg-[#1C2233] rounded-lg">
                    <Mic className={`w-5 h-5 ${tempSettings.autoCapture ? 'text-green-500' : 'text-[#9CA3AF]'}`} />
                  </div>
                  <div>
                    <p className="font-medium text-[#F9FAFB]">Enable Auto Recording</p>
                    <p className="text-sm text-[#9CA3AF]">
                      {tempSettings.autoCapture
                        ? 'Audio will be recorded automatically'
                        : 'Auto recording is disabled'}
                    </p>
                  </div>
                </div>
                <Switch
                  checked={tempSettings.autoCapture}
                  onCheckedChange={(checked) => setTempSettings({ ...tempSettings, autoCapture: checked })}
                  className="data-[state=checked]:bg-[#4C8BF5] data-[state=unchecked]:bg-[#1C2233]"
                />
              </div>

              {/* Capture Interval */}
              <div className="space-y-4">
                <div className="flex items-center gap-3">
                  <div className="p-2 bg-[#1C2233] rounded-lg">
                    <Clock className="w-5 h-5 text-[#4C8BF5]" />
                  </div>
                  <div>
                    <p className="font-medium text-[#F9FAFB]">Capture Interval</p>
                    <p className="text-sm text-[#9CA3AF]">
                      Record every {formatTimeForDisplay(tempSettings.captureInterval)}
                    </p>
                  </div>
                </div>
                <div className="px-4">
                  <Slider
                    min={30}
                    max={600}
                    step={30}
                    value={[tempSettings.captureInterval]}
                    onValueChange={(value: number[]) =>
                      setTempSettings({ ...tempSettings, captureInterval: value[0] })
                    }
                  />
                  <div className="flex justify-between text-xs text-[#9CA3AF] mt-2">
                    <span>30s</span>
                    <span>5m</span>
                    <span>10m</span>
                  </div>
                </div>
              </div>

              {/* Recording Duration */}
              <div className="space-y-4">
                <div className="flex items-center gap-3">
                  <div className="p-2 bg-[#1C2233] rounded-lg">
                    <Square className="w-5 h-5 text-[#4C8BF5]" />
                  </div>
                  <div>
                    <p className="font-medium text-[#F9FAFB]">Recording Duration</p>
                    <p className="text-sm text-[#9CA3AF]">
                      {formatTimeForDisplay(tempSettings.recordingDuration)} per recording
                    </p>
                  </div>
                </div>
                <div className="px-4">
                  <Slider
                    min={5}
                    max={getMaxRecordingDuration()}
                    step={5}
                    value={[tempSettings.recordingDuration]}
                    onValueChange={(value: number[]) =>
                      setTempSettings({ ...tempSettings, recordingDuration: value[0] })
                    }
                  />
                  <div className="flex justify-between text-xs text-[#9CA3AF] mt-2">
                    <span>5s</span>
                    <span>{formatTimeForDisplay(Math.floor(getMaxRecordingDuration() / 2))}</span>
                    <span>{formatTimeForDisplay(getMaxRecordingDuration())}</span>
                  </div>
                </div>
                <p className="text-xs text-[#9CA3AF] italic">
                  Maximum recording length is limited to 80% of capture interval.
                </p>
              </div>
            </div>
            {/* Action Buttons */}
            <div className="flex justify-end gap-4 pt-4">
              <Button onClick={() => setShowSettings(false)} className="btn-secondary">
                Cancel
              </Button>
              <Button onClick={saveSettings} className="btn-primary" disabled={isSavingSettings}>
                {isSavingSettings ? 'Saving...' : 'Save Settings'}
              </Button>
            </div>
          </div>
        </div>
      )}

      {/* Recording Controls */}
      <div className="bg-[#151926] rounded-lg p-6 mb-6">
        <div className="flex items-center gap-2 mb-5">
          <Mic className="h-6 w-6 text-blue-400" />
          <h3 className="text-lg font-semibold text-white">Recording Controls</h3>
        </div>

        <div className="space-y-3 mb-5">
          <div className="flex items-center">
            <span className="text-gray-300 w-14">Status:</span>
            <span
              className={cn(
                isRecording ? (isPaused ? 'text-yellow-500' : 'text-red-500') : 'text-gray-400',
                'font-medium'
              )}
            >
              {isRecording ? (isPaused ? 'Paused' : 'Recording in progress') : 'Inactive'}
            </span>
          </div>
        </div>

        <div>
          {!isRecording ? (
            <div className="flex gap-3">
              <Button onClick={handleStartRecording} className="btn-primary flex items-center gap-2">
                <Play className="h-4 w-4" />
                <span>Start Recording</span>
              </Button>
            </div>
          ) : (
            <div className="flex gap-3">
              <Button
                onClick={handlePauseRecording}
                className="btn-secondary flex items-center gap-2"
              >
                {isPaused ? <Play className="h-4 w-4" /> : <Pause className="h-4 w-4" />}
                <span>{isPaused ? 'Resume' : 'Pause'}</span>
              </Button>
              <Button
                onClick={handleStopRecording}
                className="btn-secondary flex items-center gap-2"
              >
                <Square className="h-4 w-4" />
                <span>Stop</span>
              </Button>
            </div>
          )}
        </div>
      </div>

      {/* Audio Recordings List */}
      <div className="bg-[#151926] rounded-lg">
        <div className="flex justify-between items-center p-6 border-b border-gray-800">
          <h3 className="text-lg font-semibold">Audio Recordings</h3>
          <Button onClick={fetchRecordings} className="btn-primary">
                <svg 
                  xmlns="http://www.w3.org/2000/svg" 
                  className="h-5 w-5 mr-2" 
                  fill="none" 
                  viewBox="0 0 24 24" 
                  stroke="currentColor"
                >
                  <path 
                    strokeLinecap="round" 
                    strokeLinejoin="round" 
                    strokeWidth={2} 
                    d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" 
                  />
                </svg>
                Refresh
              </Button>
        </div>

        <div className="p-10">
          {isLoading ? (
            <div className="flex justify-center">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-400"></div>
            </div>
          ) : recordings.length > 0 ? (
            <div className="space-y-2 max-h-64 overflow-y-auto">
              {recordings.map((recording, index) => (
                <div key={index} className="flex justify-between items-center p-3 bg-[#1b1f2e] rounded-lg">
                  <div className="flex flex-col">
                    <span className="text-sm font-medium">{recording.filename}</span>
                    <span className="text-xs text-gray-400">
                      {recording.created_at} • {recording.duration} sec •{' '}
                      {Math.round(recording.size / 1024)} KB
                    </span>
                  </div>
                  <Button variant="ghost" size="icon" className="text-gray-400 hover:text-white">
                    &#8681;
                  </Button>
                </div>
              ))}
            </div>
          ) : (
            <div className="flex flex-col items-center justify-center">
              <div className="text-gray-400 text-center">No recordings found</div>
            </div>
          )}
        </div>
      </div>

      {errorMessage && (
        <div className="mt-4 p-3 bg-red-500/20 text-red-400 rounded-lg text-sm">
          {errorMessage}
        </div>
      )}
    </div>
  );
};

export default MicrophoneDashboard;
