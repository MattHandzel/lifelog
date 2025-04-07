import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button } from './ui/button';
import { Camera, X, Settings, Power, Clock, ArrowUpDown, RefreshCw } from 'lucide-react';
import { Slider } from './ui/slider';
import { Switch } from './ui/switch';
import axios from 'axios';

// Server API endpoint from environment variable
const API_BASE_URL = import.meta.env.VITE_API_BASE_URL;

interface Screenshot {
  id: number;
  timestamp: number;
  path: string;
  dataUrl?: string; // Optional data URL for direct image data
}

interface ScreenSettings {
  enabled: boolean;
  interval: number;
  output_dir: string;
  program: string;
  timestamp_format: string;
}

export default function ScreenDashboard() {
  const [screenshots, setScreenshots] = useState<Screenshot[]>([]);
  const [currentPage, setCurrentPage] = useState(1);
  const [isLoading, setIsLoading] = useState(false);
  const [selectedImage, setSelectedImage] = useState<string | null>(null);
  const [selectedScreenshot, setSelectedScreenshot] = useState<Screenshot | null>(null);
  const [totalPages, setTotalPages] = useState(1);
  const [showSettings, setShowSettings] = useState(false);
  const [settings, setSettings] = useState<ScreenSettings | null>(null);
  const [isLoadingSettings, setIsLoadingSettings] = useState(false);
  const [isSavingSettings, setIsSavingSettings] = useState(false);
  const [tempInterval, setTempInterval] = useState(60);
  const [tempEnabled, setTempEnabled] = useState(true);
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('asc');
  const [autoRefresh, setAutoRefresh] = useState(false);
  
  // Use a ref to track the refresh interval
  const refreshIntervalRef = useRef<number>();
  
  const pageSize = 9;

  // Initial load effect
  useEffect(() => {
    loadScreenshots();
    loadSettings();
    
    // Load auto-refresh setting from localStorage
    const savedAutoRefresh = localStorage.getItem('screenshots_auto_refresh');
    if (savedAutoRefresh !== null) {
      setAutoRefresh(savedAutoRefresh === 'true');
    }
  }, []);
  
  // Handle page changes
  useEffect(() => {
    loadScreenshots();
  }, [currentPage]);
  
  // Handle auto-refresh setup/teardown
  useEffect(() => {
    // Clear any existing interval first
    if (refreshIntervalRef.current) {
      console.log('Clearing existing auto-refresh interval');
      clearInterval(refreshIntervalRef.current);
      refreshIntervalRef.current = undefined;
    }
    
    // Only create a new interval if auto-refresh is enabled and we have settings
    if (autoRefresh && settings) {
      console.log(`Setting up auto-refresh interval: ${settings.interval} seconds`);
      refreshIntervalRef.current = window.setInterval(() => {
        console.log('Auto-refreshing screenshots...');
        loadScreenshots();
      }, settings.interval * 1000);
    }
    
    // Save auto-refresh setting to localStorage
    localStorage.setItem('screenshots_auto_refresh', autoRefresh.toString());
    
    // Cleanup function to clear interval on unmount or when dependencies change
    return () => {
      if (refreshIntervalRef.current) {
        console.log('Cleanup: clearing auto-refresh interval');
        clearInterval(refreshIntervalRef.current);
        refreshIntervalRef.current = undefined;
      }
    };
  }, [autoRefresh, settings?.interval]);

  async function loadScreenshots() {
    setIsLoading(true);
    try {
      // Use server API to get frames
      const response = await axios.get(`${API_BASE_URL}/api/logger/screen/data`, {
        params: {
          page: currentPage,
          page_size: pageSize,
          limit: pageSize,
          ...(sortOrder === 'desc' && { filter: "ORDER BY timestamp DESC" }),
          ...(sortOrder === 'asc' && { filter: "ORDER BY timestamp ASC" })
        }
      });
      
      console.log("Screen frames loaded:", response.data);
      
      // Load image data for each frame
      const screenshotsWithData = await Promise.all(
        response.data.map(async (screenshot: any) => {
          try {
            // For images stored as files, we need a separate API to get the image data
            const imageResponse = await axios.get(`${API_BASE_URL}/api/logger/screen/files/${screenshot.path}`, {
              responseType: 'blob'
            });
            
            const dataUrl = URL.createObjectURL(imageResponse.data);
            return { 
              timestamp: screenshot.timestamp,
              path: screenshot.path,
              width: screenshot.width || 0,
              height: screenshot.height || 0,
              dataUrl 
            };
          } catch (error) {
            console.error(`Failed to load data for screenshot ${screenshot.path}:`, error);
            return screenshot;
          }
        })
      );
      
      setScreenshots(screenshotsWithData);
      
      // Calculate total pages based on header or data size
      const totalCount = response.headers['x-total-count'] 
        ? parseInt(response.headers['x-total-count']) 
        : response.data.length === pageSize ? (currentPage + 1) * pageSize : currentPage * pageSize;
        
      setTotalPages(Math.ceil(totalCount / pageSize));
    } catch (error) {
      console.error('Failed to load screenshots:', error);
    } finally {
      setIsLoading(false);
    }
  }

  async function loadSettings() {
    setIsLoadingSettings(true);
    try {
      // Use server API to get screen configuration
      const response = await axios.get(`${API_BASE_URL}/api/logger/screen/config`);
      const apiSettings = response.data;
      
      // Map server config to our settings format
      const settings: ScreenSettings = {
        enabled: apiSettings.enabled,
        interval: apiSettings.interval,
        output_dir: apiSettings.output_dir || '',
        program: apiSettings.program || '',
        timestamp_format: apiSettings.timestamp_format || '%Y-%m-%d_%H-%M-%S'
      };
      
      setSettings(settings);
      setTempInterval(settings.interval);
      setTempEnabled(settings.enabled);
    } catch (error) {
      console.error('Failed to load screen settings:', error);
    } finally {
      setIsLoadingSettings(false);
    }
  }

  async function saveSettings() {
    if (!settings) return;
    
    // Temporarily disable auto-refresh during save to avoid issues
    const wasAutoRefreshEnabled = autoRefresh;
    if (wasAutoRefreshEnabled) {
      console.log('Temporarily disabling auto-refresh during settings save');
      setAutoRefresh(false);
      
      // Force immediate interval cleanup
      if (refreshIntervalRef.current) {
        clearInterval(refreshIntervalRef.current);
        refreshIntervalRef.current = undefined;
      }
    }
    
    setIsSavingSettings(true);
    try {
      console.log('Updating screen settings...');
      // Use server API to update screen configuration
      await axios.put(`${API_BASE_URL}/api/logger/screen/config`, {
        enabled: tempEnabled,
        interval: tempInterval
      });
      
      // Update local settings state
      setSettings({
        ...settings,
        enabled: tempEnabled,
        interval: tempInterval
      });
      
      console.log(`Screen settings updated: enabled=${tempEnabled}, interval=${tempInterval}s`);
      
      // If enabled, restart the screen logger via server API
      if (tempEnabled) {
        try {
          console.log('Restarting screen logger...');
          await axios.post(`${API_BASE_URL}/api/logger/screen/start`);
          console.log('Screen logger restarted');
          
          // After a short delay, refresh the screenshots to show the newly captured ones
          setTimeout(() => {
            loadScreenshots();
          }, 2000);
        } catch (restartError) {
          console.error('Failed to restart screen logger:', restartError);
        }
      } else {
        // Stop the logger if disabled
        try {
          await axios.post(`${API_BASE_URL}/api/logger/screen/stop`);
        } catch (stopError) {
          console.error('Failed to stop screen logger:', stopError);
        }
      }
      
      setShowSettings(false);
    } catch (error) {
      console.error('Failed to save settings:', error);
      alert('Failed to save settings: ' + error);
    } finally {
      setIsSavingSettings(false);
      
      // Re-enable auto-refresh after a short delay if it was enabled before
      if (wasAutoRefreshEnabled) {
        console.log('Restoring auto-refresh after settings save');
        setTimeout(() => setAutoRefresh(true), 500);
      }
    }
  }

  function formatTimestamp(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleString(undefined, {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
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

  function handleScreenshotClick(screenshot: Screenshot) {
    console.log("Opening screenshot:", screenshot);
    const cleanPath = screenshot.path.replace(/^\/+/, '');
    setSelectedImage(cleanPath);
    setSelectedScreenshot(screenshot);
  }

  function closeModal() {
    setSelectedImage(null);
    setSelectedScreenshot(null);
  }

  function toggleSettings() {
    setShowSettings(!showSettings);
    if (!showSettings && settings) {
      setTempInterval(settings.interval);
      setTempEnabled(settings.enabled);
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

  // NEW: Toggle function to switch between ascending and descending order
  function toggleSortOrder() {
    setSortOrder((prev) => (prev === 'asc' ? 'desc' : 'asc'));
  }

  // NEW: Sort the screenshots by timestamp according to sortOrder
  const sortedScreenshots = [...screenshots].sort((a, b) => {
    return sortOrder === 'asc'
      ? a.timestamp - b.timestamp
      : b.timestamp - a.timestamp;
  });

  return (
    <div className="p-6 md:p-8 space-y-6">
      <div className="mb-8">
        <div className="flex items-center justify-between mb-2">
          <div className="flex items-center gap-3">
            <Camera className="w-8 h-8 text-[#4C8BF5]" />
            <h1 className="title">Screenshots</h1>
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
        <p className="subtitle">Browse captured screenshots</p>
      </div>

      {/* Settings Panel */}
      {showSettings && (
        <div className="card mb-8">
          <div className="p-6">
            <h2 className="text-lg font-medium text-[#F9FAFB] mb-6">Screenshot Settings</h2>
            <div className="space-y-6">
              {/* Enable/Disable Setting */}
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <div className="p-2 bg-[#1C2233] rounded-lg">
                    <Power className={`w-5 h-5 ${tempEnabled ? 'text-green-500' : 'text-[#9CA3AF]'}`} />
                  </div>
                  <div>
                    <p className="font-medium text-[#F9FAFB]">Enable Screenshots</p>
                    <p className="text-sm text-[#9CA3AF]">
                      {tempEnabled ? 'Screenshots are being captured' : 'Screenshot capture is paused'}
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
                      Take a screenshot every {formatIntervalDisplay(tempInterval)}
                    </p>
                  </div>
                </div>
                
                <div className="px-4">
                  <Slider 
                    min={30}
                    max={600}
                    step={30}
                    value={[tempInterval]} 
                    onValueChange={(values: number[]) => setTempInterval(values[0])}
                  />
                  <div className="flex justify-between text-xs text-[#9CA3AF] mt-2">
                    <span>30s</span>
                    <span>5m</span>
                    <span>10m</span>
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
              <Button onClick={loadScreenshots}>
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

              <Button 
                onClick={toggleSortOrder} 
                variant="secondary"
              >
                <ArrowUpDown className="w-4 h-4 mr-2" />
                {sortOrder === 'asc' ? 'Oldest First' : 'Newest First'}
              </Button>

              <div className={`flex items-center gap-2 px-3 py-2 rounded-lg ${autoRefresh ? 'bg-[#4C8BF5]/20 border border-[#4C8BF5]/40' : 'bg-[#1C2233]'}`}>
                <input
                  type="checkbox"
                  id="auto-refresh"
                  checked={autoRefresh}
                  onChange={(e) => setAutoRefresh(e.target.checked)}
                  className="h-4 w-4 text-[#4C8BF5] rounded focus:ring-2 focus:ring-[#4C8BF5]/20 bg-[#232B3D] border-[#2A3142]"
                />
                <div className="flex items-center gap-1">
                  <label htmlFor="auto-refresh" className="text-sm font-medium text-[#F9FAFB]">
                    Auto-refresh ({settings?.interval ? formatIntervalDisplay(settings.interval) : '30s'})
                  </label>
                  {autoRefresh && (
                    <RefreshCw className="w-3 h-3 text-[#4C8BF5] animate-spin ml-1" style={{ animationDuration: '2s' }} />
                  )}
                </div>
              </div>
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
              <p>Loading screenshots...</p>
            </div>
          ) : (
            <>
              {sortedScreenshots.length === 0 ? (
                <div className="flex flex-col items-center justify-center py-12 text-[#9CA3AF]">
                  <Camera className="w-12 h-12 mb-4" />
                  <p className="mb-2">No screenshots found</p>
                  <p className="text-sm max-w-md text-center">
                    {settings?.enabled 
                      ? "Screenshots are being captured automatically. They will appear here soon."
                      : "Screenshots are currently disabled. Enable them in settings."}
                  </p>
                </div>
              ) : (
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                  {sortedScreenshots.map((screenshot, index) => {
                    const pageCount = sortedScreenshots.length;
                    const displayNumber =
                      sortOrder === 'asc'
                        ? (currentPage - 1) * pageSize + index + 1
                        : (currentPage - 1) * pageSize + (pageCount - index);
                    return (
                      <div 
                        key={screenshot.id} 
                        className="card card-hover overflow-hidden cursor-pointer group"
                        onClick={() => handleScreenshotClick(screenshot)}
                      >
                        <div className="aspect-video bg-[#1C2233] relative overflow-hidden">
                          {screenshot.dataUrl ? (
                            <img 
                              src={screenshot.dataUrl} 
                              alt={`Screenshot from ${formatTimestamp(screenshot.timestamp)}`} 
                              className="w-full h-full object-cover transition-transform duration-300 group-hover:scale-105"
                              onLoad={() => console.log("Thumbnail loaded from data URL")}
                            />
                          ) : (
                            <div className="w-full h-full flex items-center justify-center">
                              <button 
                                className="bg-[#1C2233] p-3 rounded-lg text-[#4C8BF5] border border-[#2A3142]"
                                onClick={(e) => {
                                  e.stopPropagation();
                                  // Manually try to load the data for this screenshot
                                  (async () => {
                                    try {
                                      const dataUrl = await axios.get(`${API_BASE_URL}/api/logger/screen/files/${screenshot.path}`, {
                                        responseType: 'blob'
                                      });
                                      
                                      const url = URL.createObjectURL(dataUrl.data);
                                      // Update the screenshot with the data URL
                                      setScreenshots(prevScreenshots => 
                                        prevScreenshots.map(s => 
                                          s.id === screenshot.id ? {...s, dataUrl: url} : s
                                        )
                                      );
                                    } catch (error) {
                                      console.error(`Failed to manually load data for screenshot ${screenshot.path}:`, error);
                                    }
                                  })();
                                }}
                              >
                                Load Image
                              </button>
                            </div>
                          )}
                          <div className="absolute bottom-0 left-0 w-full p-2 bg-gradient-to-t from-black/70 to-transparent text-white opacity-0 group-hover:opacity-100 transition-opacity">
                            <p className="text-sm truncate">Click to view fullsize</p>
                          </div>
                        </div>
                        <div className="p-3">
                          <div className="flex items-center justify-between">
                            <div className="text-sm font-medium text-[#F9FAFB]">
                              {formatTimestamp(screenshot.timestamp)}
                            </div>
                            <div className="bg-[#4C8BF5]/10 text-[#4C8BF5] text-xs px-2 py-1 rounded-full">
                              #{displayNumber}
                            </div>
                          </div>
                          <div className="text-xs text-gray-400 mt-1 truncate">
                            {screenshot.path}
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
            <div className="absolute top-0 left-0 z-50 bg-black/50 text-white text-xs p-1">
              Image path: {selectedImage}
            </div>
            {selectedScreenshot?.dataUrl ? (
              <img 
                src={selectedScreenshot.dataUrl} 
                alt="Full size screenshot" 
                className="w-full h-auto rounded-lg shadow-2xl"
                onLoad={() => console.log("Full size image loaded successfully from data URL")}
              />
            ) : (
              <div className="w-full h-[calc(100vh-200px)] flex items-center justify-center bg-[#1C2233] rounded-lg">
                <div className="text-center p-8">
                  <span className="block text-[#9CA3AF] mb-4">Failed to load full-size image.</span>
                  <Button
                    onClick={async () => {
                      if (selectedScreenshot && selectedImage) {
                        try {
                          const dataUrl = await axios.get(`${API_BASE_URL}/api/logger/screen/files/${selectedImage}`, {
                            responseType: 'blob'
                          });
                          
                          const url = URL.createObjectURL(dataUrl.data);
                          // Update the screenshot with the data URL
                          setSelectedScreenshot({...selectedScreenshot, dataUrl: url});
                          
                          // Also update in the main list
                          setScreenshots(prevScreenshots => 
                            prevScreenshots.map(s => 
                              s.id === selectedScreenshot.id ? {...s, dataUrl: url} : s
                            )
                          );
                        } catch (error) {
                          console.error(`Failed to manually load data for fullsize image:`, error);
                        }
                      }
                    }}
                  >
                    Try Again
                  </Button>
                </div>
              </div>
            )}
            <Button 
              onClick={closeModal}
              variant="secondary"
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