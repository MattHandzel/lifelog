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

const MicrophoneDashboard: React.FC = function (): JSX.Element {
  // Dashboard states
  const [isRecording] = useState<boolean>(false);
  const [isPaused] = useState<boolean>(false);
  const [recordings, setRecordings] = useState<AudioFile[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  // Settings state from the backend
  const [settings, _setSettings] = useState<MicrophoneSettings>({
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

  useEffect(function () {
    loadSettings();
    fetchRecordings();

    return function () {
      if (statusCheckInterval.current) {
        clearInterval(statusCheckInterval.current);
      }
    };
  }, []);

  // Ensure recording duration is less than capture interval (80% max)
  useEffect(function () {
    if (tempSettings.recordingDuration >= tempSettings.captureInterval) {
      const newDuration = Math.floor(tempSettings.captureInterval * 0.8);
      setTempSettings(function (prev) { return { ...prev, recordingDuration: newDuration }; });
    }
  }, [tempSettings.captureInterval]);

  async function loadSettings(): Promise<void> {
    console.warn('Microphone settings: not yet implemented via gRPC');
    setErrorMessage(null);
  }

  async function saveSettings(): Promise<void> {
    console.warn('Microphone settings save: not yet implemented via gRPC');
    setIsSavingSettings(false);
  }

  async function fetchRecordings(): Promise<void> {
    console.warn('Microphone recordings: not yet implemented via gRPC');
    setRecordings([]);
    setIsLoading(false);
  }

  async function handleStartRecording(): Promise<void> {
    console.warn('Microphone start recording: not yet implemented via gRPC');
    setErrorMessage(null);
  }

  async function handlePauseRecording(): Promise<void> {
    console.warn('Microphone pause/resume recording: not yet implemented via gRPC');
    setErrorMessage(null);
  }

  async function handleStopRecording(): Promise<void> {
    console.warn('Microphone stop recording: not yet implemented via gRPC');
    setErrorMessage(null);
  }

  async function handleOpenTerminalForRecording(): Promise<void> {
    try {
      setErrorMessage(null);
      // This can still use invoke as it's a Tauri-specific action
      await invoke('open_terminal_for_audio', {
        directory: settings.output_dir,
      });
    } catch (error) {
      console.error('Failed to open terminal:', error);
      setErrorMessage(`Failed to open terminal: ${error}`);
    }
  }

  function formatTimeForDisplay(seconds: number): string {
    if (seconds < 60) {
      return `${seconds} seconds`;
    } else if (seconds === 60) {
      return "1 minute";
    } else if (seconds < 3600) {
      const minutes = Math.floor(seconds / 60);
      const remainingSeconds = seconds % 60;
      return remainingSeconds > 0 ? `${minutes}m ${remainingSeconds}s` : `${minutes} minutes`;
    } else {
      const hours = Math.floor(seconds / 3600);
      const minutes = Math.floor((seconds % 3600) / 60);
      return minutes > 0 ? `${hours}h ${minutes}m` : `${hours} hours`;
    }
  }

  function getMaxRecordingDuration(): number {
    return Math.floor(tempSettings.captureInterval * 0.8);
  }

  async function handlePlayRecording(_audioFile: AudioFile): Promise<void> {
    console.warn('Microphone play recording: not yet implemented via gRPC');
  }

  function formatFileSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  return (
    <div className="p-6 md:p-8 space-y-6">
      {/* Header section */}
      <div className="mb-8">
        <div className="flex items-center justify-between mb-2">
          <div className="flex items-center gap-3">
            <Mic className="w-8 h-8 text-[#4C8BF5]" />
            <h1 className="title">Microphone</h1>
          </div>
          <Button 
            onClick={() => setShowSettings(!showSettings)}
            variant="secondary"
            className="flex items-center gap-2"
          >
            <Settings className="w-4 h-4" />
            Settings
          </Button>
        </div>
        <p className="subtitle">Record and manage audio from your microphone</p>
      </div>

      {/* Error message display */}
      {errorMessage && (
        <div className="bg-red-500/10 border border-red-500/30 text-red-500 p-4 rounded-lg mb-6">
          <p>{errorMessage}</p>
        </div>
      )}

      {/* Settings panel */}
      {showSettings && (
        <div className="card mb-8">
          <div className="p-6">
            <h2 className="text-lg font-medium text-[#F9FAFB] mb-6">Microphone Settings</h2>
            <div className="space-y-6">
              {/* Auto-capture toggle */}
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <div className={`p-2 bg-[#1C2233] rounded-lg ${tempSettings.autoCapture ? 'text-green-500' : 'text-[#9CA3AF]'}`}>
                    <Mic className="w-5 h-5" />
                  </div>
                  <div>
                    <p className="font-medium text-[#F9FAFB]">Auto-Recording</p>
                    <p className="text-sm text-[#9CA3AF]">
                      {tempSettings.autoCapture 
                        ? `Recording automatically every ${formatTimeForDisplay(tempSettings.captureInterval)}` 
                        : 'Automatic recording is disabled'}
                    </p>
                  </div>
                </div>
                <Switch 
                  checked={tempSettings.autoCapture} 
                  onCheckedChange={(checked) => setTempSettings({...tempSettings, autoCapture: checked})}
                  className="data-[state=checked]:bg-[#4C8BF5]"
                />
              </div>

              {/* Capture interval setting */}
              {tempSettings.autoCapture && (
                <div className="space-y-4">
                  <div className="flex items-center gap-3">
                    <div className="p-2 bg-[#1C2233] rounded-lg">
                      <Clock className="w-5 h-5 text-[#4C8BF5]" />
                    </div>
                    <div>
                      <p className="font-medium text-[#F9FAFB]">Capture Interval</p>
                      <p className="text-sm text-[#9CA3AF]">
                        Record automatically every {formatTimeForDisplay(tempSettings.captureInterval)}
                      </p>
                    </div>
                  </div>
                  
                  <div className="px-4">
                    <Slider 
                      min={60}
                      max={3600}
                      step={30}
                      value={[tempSettings.captureInterval]} 
                      onValueChange={(vals) => setTempSettings({...tempSettings, captureInterval: vals[0]})}
                    />
                    <div className="flex justify-between text-xs text-[#9CA3AF] mt-1">
                      <span>1m</span>
                      <span>30m</span>
                      <span>60m</span>
                    </div>
                  </div>
                </div>
              )}

              {/* Recording duration setting */}
              {tempSettings.autoCapture && (
                <div className="space-y-4">
                  <div className="flex items-center gap-3">
                    <div className="p-2 bg-[#1C2233] rounded-lg">
                      <Clock className="w-5 h-5 text-[#4C8BF5]" />
                    </div>
                    <div>
                      <p className="font-medium text-[#F9FAFB]">Recording Duration</p>
                      <p className="text-sm text-[#9CA3AF]">
                        Each recording lasts {formatTimeForDisplay(tempSettings.recordingDuration)}
                      </p>
                    </div>
                  </div>
                  
                  <div className="px-4">
                    <Slider 
                      min={5}
                      max={getMaxRecordingDuration()}
                      step={5}
                      value={[tempSettings.recordingDuration]} 
                      onValueChange={(vals) => setTempSettings({...tempSettings, recordingDuration: vals[0]})}
                    />
                    <div className="flex justify-between text-xs text-[#9CA3AF] mt-1">
                      <span>5s</span>
                      <span>{formatTimeForDisplay(Math.floor(getMaxRecordingDuration() / 2))}</span>
                      <span>{formatTimeForDisplay(getMaxRecordingDuration())}</span>
                    </div>
                  </div>
                </div>
              )}

              {/* Actions */}
              <div className="flex justify-end gap-4 pt-4 border-t border-[#2A3142]">
                <Button
                  onClick={() => setShowSettings(false)}
                  variant="secondary"
                  disabled={isSavingSettings}
                >
                  Cancel
                </Button>
                <Button
                  onClick={saveSettings}
                  variant="default"
                  disabled={isSavingSettings}
                >
                  {isSavingSettings ? (
                    <>
                      <svg className="animate-spin -ml-1 mr-2 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                        <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                        <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                      </svg>
                      Saving...
                    </>
                  ) : 'Save Settings'}
                </Button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Manual recording controls */}
      <div className="card mb-8">
        <div className="p-6">
          <h2 className="text-lg font-medium text-[#F9FAFB] mb-4">Manual Recording</h2>
          
          <div className="flex flex-wrap gap-4 mb-6">
            {!isRecording ? (
              <Button
                onClick={handleStartRecording}
                variant="default"
                className="flex items-center gap-2"
              >
                <Play className="w-4 h-4" />
                Start Recording
              </Button>
            ) : (
              <>
                <Button
                  onClick={handlePauseRecording}
                  className={cn("flex items-center gap-2", 
                    isPaused && "bg-[#4C8BF5]/10 text-[#4C8BF5] border-[#4C8BF5]/30"
                  )}
                  variant="secondary"
                >
                  {isPaused ? (
                    <>
                      <Play className="w-4 h-4" />
                      Resume
                    </>
                  ) : (
                    <>
                      <Pause className="w-4 h-4" />
                      Pause
                    </>
                  )}
                </Button>
                
                <Button
                  onClick={handleStopRecording}
                  variant="secondary"
                  className="flex items-center gap-2 bg-red-500/10 hover:bg-red-500/20 text-red-500 border-red-500/30"
                >
                  <Square className="w-4 h-4" />
                  Stop
                </Button>
              </>
            )}
            
            <Button
              onClick={handleOpenTerminalForRecording}
              variant="secondary"
              className="flex items-center gap-2"
              title="Open terminal in the recordings directory"
            >
              <ExternalLink className="w-4 h-4" />
              Open Directory
            </Button>
            
            <Button
              onClick={fetchRecordings}
              variant="secondary"
              className="flex items-center gap-2"
            >
              <svg className="w-4 h-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
              Refresh
            </Button>
          </div>
          
          {isRecording && (
            <div className="bg-[#1C2233] p-3 rounded-lg flex items-center mb-4">
              <div className={cn(
                "w-3 h-3 rounded-full mr-3", 
                isPaused ? "bg-yellow-500" : "bg-red-500 animate-pulse"
              )}></div>
              <p className="text-sm">
                {isPaused ? "Recording paused" : "Recording in progress..."}
              </p>
            </div>
          )}
          
          {settings.enabled && (
            <div className="bg-[#1C2233] p-3 rounded-lg flex items-center mb-4">
              <div className="w-3 h-3 rounded-full bg-green-500 mr-3"></div>
              <p className="text-sm">
                Auto-recording is active. Capturing for {formatTimeForDisplay(settings.chunk_duration_secs)} every {formatTimeForDisplay(settings.capture_interval_secs)}.
              </p>
            </div>
          )}
        </div>
      </div>

      {/* Recordings list */}
      <div className="card">
        <div className="p-6">
          <h2 className="text-lg font-medium text-[#F9FAFB] mb-4">Recent Recordings</h2>
          
          {isLoading ? (
            <div className="flex flex-col items-center justify-center py-12">
              <svg className="animate-spin h-10 w-10 text-[#4C8BF5] mb-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              <p className="text-[#9CA3AF]">Loading recordings...</p>
            </div>
          ) : recordings.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-12 text-[#9CA3AF]">
              <Mic className="w-12 h-12 mb-4" />
              <p className="mb-2">No recordings found</p>
              <p className="text-sm text-center max-w-md">
                Start a manual recording or enable auto-recording to capture audio.
              </p>
            </div>
          ) : (
            <div className="space-y-2">
              {recordings.map((recording, index) => (
                <div key={index} className="bg-[#1C2233] rounded-lg p-4 flex flex-col sm:flex-row sm:items-center gap-4">
                  <div className="flex-1">
                    <div className="flex items-center gap-2 mb-1">
                      <Mic className="w-4 h-4 text-[#4C8BF5]" />
                      <p className="font-medium text-[#F9FAFB] truncate">{recording.filename}</p>
                    </div>
                    <div className="flex flex-wrap text-xs text-[#9CA3AF] gap-x-4">
                      <span>Duration: {formatTimeForDisplay(recording.duration)}</span>
                      <span>Size: {formatFileSize(recording.size)}</span>
                      <span>Created: {new Date(recording.created_at).toLocaleString()}</span>
                    </div>
                  </div>
                  <div className="flex gap-2">
                    <Button 
                      onClick={() => handlePlayRecording(recording)}
                      variant="default"
                      title="Play recording"
                    >
                      <Play className="w-4 h-4" />
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default MicrophoneDashboard;
