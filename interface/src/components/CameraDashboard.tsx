import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button } from './ui/button';
import { Settings, Power, Clock, X, RefreshCcw, Camera } from 'lucide-react';
import { Slider } from './ui/slider';
import { Switch } from './ui/switch';
import axios from 'axios';
import { Card, CardContent } from './ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from './ui/tabs';
import { Trash, ArrowUpDown } from 'lucide-react';

// Server API endpoint from environment variable
const API_BASE_URL = import.meta.env.VITE_API_BASE_URL;

interface CameraFrame {
  timestamp: number;
  path: string;
  width: number;
  height: number;
  dataUrl?: string;
}

interface CameraSettings {
  enabled: boolean;
  device: string;
  fps: number;
  interval: number;
  output_dir: string;
  resolution: [number, number];
  timestamp_format: string;
}

export default function CameraDashboard() {
  const [frames, setFrames] = useState<CameraFrame[]>([]);
  const [currentPage, setCurrentPage] = useState(1);
  const [isLoading, setIsLoading] = useState(false);
  const [selectedImage, setSelectedImage] = useState<string | null>(null);
  const [selectedFrame, setSelectedFrame] = useState<CameraFrame | null>(null);
  const [totalPages, setTotalPages] = useState(1);
  const [showSettings, setShowSettings] = useState(false);
  const [settings, setSettings] = useState<CameraSettings | null>(null);
  const [isLoadingSettings, setIsLoadingSettings] = useState(false);
  const [isSavingSettings, setIsSavingSettings] = useState(false);
  const [tempInterval, setTempInterval] = useState(60);
  const [tempEnabled, setTempEnabled] = useState(true);
  const [tempFps, setTempFps] = useState(30);
  const [isSupported, setIsSupported] = useState<boolean | null>(null);
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('desc');
  
  const pageSize = 9;

  useEffect(() => {
    checkCameraSupport();
    loadFrames();
    loadSettings();
    
    const refreshInterval = setInterval(() => {
      if (!showSettings) {
        loadFrames();
      }
    }, 30000);
    
    return () => clearInterval(refreshInterval);
  }, [currentPage]);

  async function checkCameraSupport() {
    try {
      // Can still use invoke for system-specific checks
      const supported = await invoke<boolean>('is_camera_supported');
      setIsSupported(supported);
    } catch (error) {
      console.error('Failed to check camera support:', error);
      setIsSupported(false);
    }
  }

  async function loadFrames() {
    setIsLoading(true);
    try {
      // Use server API to get frames
      const response = await axios.get(`${API_BASE_URL}/api/logger/camera/data`, {
        params: {
          page: currentPage,
          page_size: pageSize,
          limit: pageSize,
          // For newest first
          ...(sortOrder === 'desc' && { filter: "ORDER BY timestamp DESC" }),
          ...(sortOrder === 'asc' && { filter: "ORDER BY timestamp ASC" })
        }
      });
      
      console.log("Camera frames loaded:", response.data);
      
      // Load image data for each frame
      const framesWithData = await Promise.all(
        response.data.map(async (frame: any) => {
          try {
            // For images stored as files, we need a separate API to get the image data
            const imageResponse = await axios.get(`${API_BASE_URL}/api/logger/camera/files/${frame.path}`, {
              responseType: 'blob'
            });
            
            const dataUrl = URL.createObjectURL(imageResponse.data);
            return { 
              timestamp: frame.timestamp,
              path: frame.path,
              width: frame.width || 0,
              height: frame.height || 0,
              dataUrl 
            };
          } catch (error) {
            console.error(`Failed to load data for frame ${frame.path}:`, error);
            return {
              timestamp: frame.timestamp,
              path: frame.path,
              width: frame.width || 0,
              height: frame.height || 0
            };
          }
        })
      );
      
      setFrames(framesWithData);
      
      // Calculate total pages based on header or data size
      const totalCount = response.headers['x-total-count'] 
        ? parseInt(response.headers['x-total-count']) 
        : response.data.length === pageSize ? (currentPage + 1) * pageSize : currentPage * pageSize;
        
      setTotalPages(Math.ceil(totalCount / pageSize));
    } catch (error) {
      console.error('Failed to load camera frames:', error);
    } finally {
      setIsLoading(false);
    }
  }

  async function loadSettings() {
    setIsLoadingSettings(true);
    try {
      // Use server API to get camera configuration
      const response = await axios.get(`${API_BASE_URL}/api/logger/camera/config`);
      const apiSettings = response.data;
      
      // Map server config to our settings format
      const settings: CameraSettings = {
        enabled: apiSettings.enabled,
        interval: apiSettings.interval,
        fps: apiSettings.fps,
        device: apiSettings.device || '',
        output_dir: apiSettings.output_dir || '',
        resolution: apiSettings.resolution || [640, 480],
        timestamp_format: apiSettings.timestamp_format || '%Y-%m-%d_%H-%M-%S'
      };
      
      setSettings(settings);
      setTempInterval(settings.interval);
      setTempEnabled(settings.enabled);
      setTempFps(settings.fps);
    } catch (error) {
      console.error('Failed to load camera settings:', error);
    } finally {
      setIsLoadingSettings(false);
    }
  }

  async function saveSettings() {
    if (!settings) return;
    
    setIsSavingSettings(true);
    try {
      console.log('Updating camera settings...');
      // Use server API to update camera configuration
      await axios.put(`${API_BASE_URL}/api/logger/camera/config`, {
        enabled: tempEnabled,
        interval: tempInterval,
        fps: tempFps
      });
      
      setSettings({
        ...settings,
        enabled: tempEnabled,
        interval: tempInterval,
        fps: tempFps
      });
      
      console.log(`Camera settings updated: enabled=${tempEnabled}, interval=${tempInterval}s, fps=${tempFps}`);
      
      // If enabled, restart the camera logger via server API
      if (tempEnabled) {
        try {
          console.log('Restarting camera logger...');
          await axios.post(`${API_BASE_URL}/api/logger/camera/start`);
          console.log('Camera logger restarted');
          
          // After a short delay, refresh the frames to show the newly captured ones
          setTimeout(() => {
            loadFrames();
          }, 2000);
        } catch (restartError) {
          console.error('Failed to restart camera logger:', restartError);
        }
      } else {
        // Stop the logger if disabled
        try {
          await axios.post(`${API_BASE_URL}/api/logger/camera/stop`);
        } catch (stopError) {
          console.error('Failed to stop camera logger:', stopError);
        }
      }
      
      setShowSettings(false);
    } catch (error) {
      console.error('Failed to save settings:', error);
      alert('Failed to save settings: ' + error);
    } finally {
      setIsSavingSettings(false);
    }
  }

  async function captureFrame() {
    try {
      setIsLoading(true);
      console.log('Triggering camera capture...');
      
      // Use server API to trigger a one-off camera capture
      await axios.post(`${API_BASE_URL}/api/logger/camera/capture`);
      
      console.log('Frame captured, reloading frames...');
      await loadFrames();
    } catch (error) {
      console.error('Failed to capture frame:', error);
      alert('Failed to capture frame: ' + error);
    } finally {
      setIsLoading(false);
    }
  }

  async function restartCameraLogger() {
    try {
      setIsLoading(true);
      console.log('Restarting camera logger...');
      
      // Stop the logger first
      await axios.post(`${API_BASE_URL}/api/logger/camera/stop`);
      
      // Then start it again
      await axios.post(`${API_BASE_URL}/api/logger/camera/start`);
      
      console.log('Camera logger restarted, refreshing frames...');
      await loadFrames();
    } catch (error) {
      console.error('Failed to restart camera logger:', error);
      alert('Failed to restart camera logger: ' + error);
    } finally {
      setIsLoading(false);
    }
  }

  function formatTimestamp(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleString(undefined, {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    });
  }

  function handlePreviousPage() {
    if (currentPage > 1) {
      setCurrentPage(currentPage - 1);
    }
  }

  function handleNextPage() {
    if (currentPage < totalPages) {
      setCurrentPage(currentPage + 1);
    }
  }

  function handleFrameClick(frame: CameraFrame) {
    console.log("Opening camera frame:", frame);
    setSelectedImage(frame.dataUrl || '');
    setSelectedFrame(frame);
  }

  function closeModal() {
    setSelectedImage(null);
    setSelectedFrame(null);
  }

  function toggleSettings() {
    setShowSettings(!showSettings);
    if (!showSettings && settings) {
      setTempInterval(settings.interval);
      setTempEnabled(settings.enabled);
      setTempFps(settings.fps);
    }
  }

  function formatIntervalDisplay(seconds: number): string {
    if (seconds < 60) {
      return `${seconds} seconds`;
    } else if (seconds === 60) {
      return "1 minute";
    } else if (seconds < 3600) {
      const minutes = Math.floor(seconds / 60);
      const remainingSeconds = seconds % 60;
      return remainingSeconds === 0
        ? `${minutes} minutes`
        : `${minutes}m ${remainingSeconds}s`;
    } else {
      const hours = Math.floor(seconds / 3600);
      const minutes = Math.floor((seconds % 3600) / 60);
      return minutes === 0
        ? `${hours} hour${hours > 1 ? 's' : ''}`
        : `${hours}h ${minutes}m`;
    }
  }

  function toggleSortOrder() {
    setSortOrder((prev) => (prev === 'asc' ? 'desc' : 'asc'));
  }

  // Sort the frames by timestamp according to sortOrder
  const sortedFrames = [...frames].sort((a, b) => {
    return sortOrder === 'asc'
      ? a.timestamp - b.timestamp
      : b.timestamp - a.timestamp;
  });

  // Handle camera support state
  let unsupportedContent = null;
  if (isSupported === false) {
    unsupportedContent = (
      <div className="p-6 md:p-8 space-y-6">
        <div className="mb-8">
          <div className="flex items-center gap-3 mb-2">
            <Camera className="w-8 h-8 text-[#4C8BF5]" />
            <h1 className="title">Camera</h1>
          </div>
          <p className="subtitle">Manage camera recordings and snapshots</p>
        </div>

        <div className="card">
          <div className="p-8 flex flex-col items-center justify-center text-center">
            <div className="bg-[#232B3D] p-4 rounded-full mb-4">
              <Camera className="w-12 h-12 text-[#9CA3AF]" />
            </div>
            <h2 className="text-xl font-medium text-[#F9FAFB] mb-2">Camera Not Available</h2>
            <p className="text-[#9CA3AF] max-w-md mb-4">
              Camera functionality is not currently working on your system.
            </p>
            <div className="bg-[#232B3D] p-4 rounded-lg text-left text-[#9CA3AF] text-sm max-w-md mb-4">
              <p className="mb-2"><strong>For macOS users:</strong></p>
              <p className="mb-2">Make sure you have the 'imagesnap' utility installed:</p>
              <code className="block bg-[#1C2233] p-2 rounded mb-3">
                brew install imagesnap
              </code>
              <p className="mb-2">You may also need to grant camera permission to the application.</p>
              <p>After installing imagesnap, please restart the application.</p>
            </div>
          </div>
        </div>
      </div>
    );
  }
  
  if (unsupportedContent) {
    return unsupportedContent;
  }

  return (
    <div className="p-6 md:p-8 space-y-6">
      <div className="mb-8">
        <div className="flex items-center justify-between mb-2">
          <div className="flex items-center gap-3">
            <Camera className="w-8 h-8 text-[#4C8BF5]" />
            <h1 className="title">Camera</h1>
          </div>
          <Button 
            onClick={toggleSettings}
            variant="secondary"
            className="flex items-center gap-2"
          >
            <Settings className="w-4 h-4" />
            Settings
          </Button>
        </div>
        <p className="subtitle">Manage camera recordings and snapshots</p>
      </div>

      {/* Settings Panel */}
      {showSettings && (
        <div className="card mb-8">
          <div className="p-6">
            <h2 className="text-lg font-medium text-[#F9FAFB] mb-6">Camera Settings</h2>
            <div className="space-y-6">
              {/* Enable/Disable Setting */}
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <div className="p-2 bg-[#1C2233] rounded-lg">
                    <Power className={`w-5 h-5 ${tempEnabled ? 'text-green-500' : 'text-[#9CA3AF]'}`} />
                  </div>
                  <div>
                    <p className="font-medium text-[#F9FAFB]">Enable Camera</p>
                    <p className="text-sm text-[#9CA3AF]">
                      {tempEnabled ? 'Camera is recording periodically' : 'Camera recording is paused'}
                    </p>
                  </div>
                </div>
                <div className="flex items-center">
                  <Switch 
                    checked={tempEnabled} 
                    onCheckedChange={setTempEnabled}
                    className="data-[state=checked]:bg-[#4C8BF5] data-[state=unchecked]:bg-[#1C2233]"
                  />
                </div>
              </div>
              
              {/* Interval Setting */}
              <div className="space-y-4">
                <div className="flex items-center gap-3">
                  <div className="p-2 bg-[#1C2233] rounded-lg">
                    <Clock className="w-5 h-5 text-[#4C8BF5]" />
                  </div>
                  <div>
                    <p className="font-medium text-[#F9FAFB]">Capture Interval</p>
                    <p className="text-sm text-[#9CA3AF]">
                      Take a snapshot every {formatIntervalDisplay(tempInterval)}
                    </p>
                  </div>
                </div>
                
                <div className="px-4">
                  <Slider 
                    min={5}
                    max={600}
                    step={5}
                    value={[tempInterval]} 
                    onValueChange={(values: number[]) => setTempInterval(values[0])}
                  />
                  <div className="flex justify-between text-xs text-[#9CA3AF] mt-2">
                    <span>5s</span>
                    <span>5m</span>
                    <span>10m</span>
                  </div>
                </div>
              </div>

              {/* FPS Setting */}
              <div className="space-y-4">
                <div className="flex items-center gap-3">
                  <div className="p-2 bg-[#1C2233] rounded-lg">
                    <RefreshCcw className="w-5 h-5 text-[#4C8BF5]" />
                  </div>
                  <div>
                    <p className="font-medium text-[#F9FAFB]">Camera FPS</p>
                    <p className="text-sm text-[#9CA3AF]">
                      {tempFps} frames per second
                    </p>
                  </div>
                </div>
                
                <div className="px-4">
                  <Slider 
                    min={1}
                    max={60}
                    step={1}
                    value={[tempFps]} 
                    onValueChange={(values: number[]) => setTempFps(values[0])}
                  />
                  <div className="flex justify-between text-xs text-[#9CA3AF] mt-2">
                    <span>1 fps</span>
                    <span>30 fps</span>
                    <span>60 fps</span>
                  </div>
                </div>
              </div>
              
              {/* Actions */}
              <div className="flex justify-end gap-4 pt-4">
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
                      <svg 
                        className="animate-spin h-4 w-4 mr-2" 
                        xmlns="http://www.w3.org/2000/svg" 
                        fill="none" 
                        viewBox="0 0 24 24"
                      >
                        <circle 
                          className="opacity-25" 
                          cx="12" 
                          cy="12" 
                          r="10" 
                          stroke="currentColor" 
                          strokeWidth="4"
                        />
                        <path 
                          className="opacity-75" 
                          fill="currentColor" 
                          d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                        />
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

      <div className="card">
        <div className="p-6">
          <div className="flex flex-col sm:flex-row justify-between gap-4 mb-8">
            <div className="flex items-center gap-2">
              <Button onClick={loadFrames} variant="secondary">
                <RefreshCcw className="w-4 h-4 mr-2" />
                Refresh
              </Button>

              <Button 
                onClick={toggleSortOrder} 
                variant="secondary"
                className="flex items-center gap-2"
              >
                <svg 
                  xmlns="http://www.w3.org/2000/svg" 
                  className="h-4 w-4 mr-1" 
                  fill="none" 
                  viewBox="0 0 24 24" 
                  stroke="currentColor"
                >
                  <path 
                    strokeLinecap="round" 
                    strokeLinejoin="round" 
                    strokeWidth={2} 
                    d="M7 16V4m0 0L3 8m4-4l4 4m6 0v12m0 0l4-4m-4 4l-4-4" 
                  />
                </svg>
                {sortOrder === 'asc' ? 'Oldest First' : 'Newest First'}
              </Button>

              <Button 
                onClick={captureFrame} 
                variant="default"
                className="flex items-center gap-2"
                disabled={!settings?.enabled}
              >
                <Camera className="w-4 h-4 mr-2" />
                Capture Now
              </Button>

              <Button 
                onClick={restartCameraLogger} 
                variant="secondary"
                className="flex items-center gap-2"
                disabled={!settings?.enabled}
                title="Restart camera logger if it's not working"
              >
                <RefreshCcw className="w-4 h-4 mr-2" />
                Restart Camera
              </Button>
            </div>
            
            <div className="flex items-center gap-2">
              <Button 
                onClick={handlePreviousPage} 
                disabled={currentPage === 1 || isLoading}
                variant="secondary"
              >
                <svg 
                  xmlns="http://www.w3.org/2000/svg" 
                  className="h-5 w-5 mr-1" 
                  fill="none" 
                  viewBox="0 0 24 24" 
                  stroke="currentColor"
                >
                  <path 
                    strokeLinecap="round" 
                    strokeLinejoin="round" 
                    strokeWidth={2} 
                    d="M15 19l-7-7 7-7" 
                  />
                </svg>
                Previous
              </Button>
              
              <div className="px-3 py-1 bg-[#1C2233] rounded-lg text-sm text-[#F9FAFB]">
                Page {currentPage} of {totalPages}
              </div>
              
              <Button 
                onClick={handleNextPage} 
                disabled={currentPage >= totalPages || isLoading}
                variant="secondary"
              >
                Next
                <svg 
                  xmlns="http://www.w3.org/2000/svg" 
                  className="h-5 w-5 ml-1" 
                  fill="none" 
                  viewBox="0 0 24 24" 
                  stroke="currentColor"
                >
                  <path 
                    strokeLinecap="round" 
                    strokeLinejoin="round" 
                    strokeWidth={2} 
                    d="M9 5l7 7-7 7" 
                  />
                </svg>
              </Button>
            </div>
          </div>

          {isLoading ? (
            <div className="flex flex-col items-center justify-center py-12 text-[#9CA3AF]">
              <svg 
                className="animate-spin h-8 w-8 text-[#4C8BF5] mb-3" 
                xmlns="http://www.w3.org/2000/svg" 
                fill="none" 
                viewBox="0 0 24 24"
              >
                <circle 
                  className="opacity-25" 
                  cx="12" 
                  cy="12" 
                  r="10" 
                  stroke="currentColor" 
                  strokeWidth="4"
                />
                <path 
                  className="opacity-75" 
                  fill="currentColor" 
                  d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                />
              </svg>
              <p>Loading camera frames...</p>
            </div>
          ) : (
            <>
              {sortedFrames.length === 0 ? (
                <div className="flex flex-col items-center justify-center py-12 text-[#9CA3AF]">
                  <Camera className="w-12 h-12 mb-4" />
                  <p className="mb-2">No camera frames found</p>
                  <p className="text-sm max-w-md text-center">
                    {settings?.enabled 
                      ? "Camera is enabled. Frames will appear here once captured."
                      : "Camera is currently disabled. Enable it in settings."}
                  </p>
                </div>
              ) : (
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                  {sortedFrames.map((frame, index) => {
                    const pageCount = sortedFrames.length;
                    const displayNumber =
                      sortOrder === 'asc'
                        ? (currentPage - 1) * pageSize + index + 1
                        : (currentPage - 1) * pageSize + (pageCount - index);
                    return (
                      <div 
                        key={frame.timestamp} 
                        className="card card-hover overflow-hidden cursor-pointer group"
                        onClick={() => handleFrameClick(frame)}
                      >
                        <div className="aspect-video bg-[#1C2233] relative overflow-hidden">
                          <img 
                            src={frame.dataUrl} 
                            alt={`Camera frame from ${formatTimestamp(frame.timestamp)}`} 
                            className="w-full h-full object-cover transition-transform duration-300 group-hover:scale-105"
                            onError={(e) => {
                              console.error("Failed to load frame thumbnail");
                              e.currentTarget.src =
                                "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Crect x='3' y='3' width='18' height='18' rx='2' ry='2'/%3E%3Ccircle cx='8.5' cy='8.5' r='1.5'/%3E%3Cpolyline points='21 15 16 10 5 21'/%3E%3C/svg%3E";
                              e.currentTarget.classList.add("p-8", "opacity-50");
                            }}
                          />
                          <div className="absolute bottom-0 left-0 w-full p-2 bg-gradient-to-t from-black/70 to-transparent text-white opacity-0 group-hover:opacity-100 transition-opacity">
                            <p className="text-sm truncate">Click to view fullsize</p>
                          </div>
                        </div>
                        <div className="p-3">
                          <div className="flex items-center justify-between">
                            <div className="text-sm font-medium text-[#F9FAFB]">
                              {formatTimestamp(frame.timestamp)}
                            </div>
                            <div className="bg-[#4C8BF5]/10 text-[#4C8BF5] text-xs px-2 py-1 rounded-full">
                              #{displayNumber}
                            </div>
                          </div>
                          <div className="text-xs text-gray-400 mt-1 truncate">
                            {frame.width}×{frame.height}
                          </div>
                        </div>
                      </div>
                    );
                  })}
                </div>
              )}
            </>
          )}
        </div>
      </div>

      {/* Image Modal */}
      {selectedImage && (
        <div 
          className="fixed inset-0 bg-[#0F111A]/90 backdrop-blur-sm flex items-center justify-center z-50" 
          onClick={closeModal}
        >
          <div 
            className="relative max-w-5xl max-h-[90vh] overflow-auto rounded-lg" 
            onClick={(e) => e.stopPropagation()}
          >
            {selectedFrame && (
              <div className="absolute top-0 left-0 z-50 bg-black/50 text-white text-xs p-2">
                {formatTimestamp(selectedFrame.timestamp)} - {selectedFrame.width}×{selectedFrame.height}
              </div>
            )}
            <img 
              src={selectedImage} 
              alt="Full size camera frame" 
              className="w-full h-auto rounded-lg shadow-2xl"
              onError={(e) => {
                console.error("Failed to load full-size image");
                e.currentTarget.src =
                  "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Crect x='3' y='3' width='18' height='18' rx='2' ry='2'/%3E%3Ccircle cx='8.5' cy='8.5' r='1.5'/%3E%3Cpolyline points='21 15 16 10 5 21'/%3E%3C/svg%3E";
                e.currentTarget.classList.add("p-16", "opacity-50");
              }}
            />
            <Button 
              onClick={closeModal}
              className="absolute top-3 right-3 rounded-full p-2 bg-[#1C2233]/80 hover:bg-[#232B3D]/80 backdrop-blur-sm"
            >
              <X className="w-6 h-6 text-[#F9FAFB]" />
            </Button>
          </div>
        </div>
      )}
    </div>
  );
} 