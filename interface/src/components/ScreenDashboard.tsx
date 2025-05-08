import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button } from './ui/button';
import { Camera, X, Settings, Power, Clock, ArrowUpDown, RefreshCw } from 'lucide-react';
import { Slider } from './ui/slider';
import { Switch } from './ui/switch';
import axios from 'axios';

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL;

interface Screenshot {
  id: number;
  timestamp: number;
  path: string;
  dataUrl?: string;
}

interface ScreenConfig {
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
  const [settings, setSettings] = useState<ScreenConfig | null>(null);
  const [isLoadingSettings, setIsLoadingSettings] = useState(false);
  const [isSavingSettings, setIsSavingSettings] = useState(false);
  const [tempInterval, setTempInterval] = useState(60);
  const [tempEnabled, setTempEnabled] = useState(true);
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('asc');
  const [autoRefresh, setAutoRefresh] = useState(false);
  const refreshIntervalRef = useRef<number>();
  const pageSize = 9;

  useEffect(() => {
    loadScreenshots();
    loadSettings();

    const savedAutoRefresh = localStorage.getItem('screenshots_auto_refresh');
    if (savedAutoRefresh !== null) {
      setAutoRefresh(savedAutoRefresh === 'true');
    }
  }, []);

  useEffect(() => {
    loadScreenshots();
  }, [currentPage, sortOrder]);

  // Handle auto-refresh setup/teardown based on settings interval
  useEffect(() => {
    if (refreshIntervalRef.current) {
      clearInterval(refreshIntervalRef.current);
      refreshIntervalRef.current = undefined;
    }

    if (autoRefresh && settings && settings.enabled && settings.interval > 0) {
       console.log(`[Screen] Setting up auto-refresh interval: ${settings.interval} seconds`);
       refreshIntervalRef.current = window.setInterval(() => {
         console.log('[Screen] Auto-refreshing...');
         // Refresh only if currently on the first page and sorting by newest
         if (currentPage === 1 && sortOrder === 'desc') {
           loadScreenshots();
         } else {
           console.log("[Screen] Auto-refresh skipped (not page 1 or not newest sort)");
         }
       }, settings.interval * 1000);
    }

    localStorage.setItem('screenshots_auto_refresh', autoRefresh.toString());

    return () => {
      if (refreshIntervalRef.current) {
        clearInterval(refreshIntervalRef.current);
        refreshIntervalRef.current = undefined;
      }
    };
  }, [autoRefresh, settings?.interval, settings?.enabled, currentPage, sortOrder]);

  async function loadScreenshots() {
    setIsLoading(true);
    try {
       const response = await axios.get(`${API_BASE_URL}/api/logger/screen/data`, {
         params: {
           page: currentPage,
           page_size: pageSize,
           limit: pageSize,
           ...(sortOrder === 'desc' ? { filter: "ORDER BY timestamp DESC" } : { filter: "ORDER BY timestamp ASC" })
         }
       });

       console.log("[Screen] Frames list loaded:", response.data);

       const mappedScreenshots = response.data.map((item: any) => ({
          id: item.id ?? Math.random(),
          timestamp: item.timestamp,
          path: item.path,
        }));

      const screenshotsWithData = await Promise.all(
        mappedScreenshots.map(async (screenshot: Screenshot) => {
          try {
            const imageResponse = await axios.get(`${API_BASE_URL}/api/files/screen/${screenshot.path}`, {
              responseType: 'blob'
            });
            const dataUrl = URL.createObjectURL(imageResponse.data);
            return { ...screenshot, dataUrl };
          } catch (error) {
            console.error(`[Screen] Failed to load data for ${screenshot.path}:`, error);
            return screenshot;
          }
        })
      );

      setScreenshots(screenshotsWithData);

      const totalCountHeader = response.headers['x-total-count'];
      const totalCount = totalCountHeader ? parseInt(totalCountHeader) : screenshotsWithData.length + (currentPage * pageSize);
      setTotalPages(Math.ceil(totalCount / pageSize));

    } catch (error) {
      console.error('[Screen] Failed to load screenshots:', error);
    } finally {
      setIsLoading(false);
    }
  }

  async function loadSettings() {
    setIsLoadingSettings(true);
    try {
      console.log("[Screen] Requesting config via Tauri...");
      const result = await invoke("get_component_config", { componentName: "screen" });
      console.log("[Screen] Received config from Tauri:", result);

      if (result && typeof result === 'object') {
         const loadedSettings = result as ScreenConfig;

         if (typeof loadedSettings.enabled !== 'boolean' || typeof loadedSettings.interval !== 'number') {
            console.error("[Screen] Invalid settings format from backend.", loadedSettings);
            throw new Error("Received invalid settings format from backend.");
         }

        console.log("[Screen] Parsed settings successfully:", loadedSettings);
        setSettings(loadedSettings);
        setTempInterval(loadedSettings.interval);
        setTempEnabled(loadedSettings.enabled);
      } else if (result === null) {
          console.warn("[Screen] Backend returned null config. Using defaults.");
          const defaultSettings: ScreenConfig = {
              enabled: false,
              interval: 60,
              output_dir: "",
              program: "",
              timestamp_format: ""
          };
          setSettings(defaultSettings);
          setTempInterval(defaultSettings.interval);
          setTempEnabled(defaultSettings.enabled);
      } else {
          console.error("[Screen] Unexpected data format for settings:", result);
          throw new Error("Received unexpected data format for settings.");
      }

    } catch (error) {
      console.error('[Screen] Failed to load settings via Tauri:', error);
       const defaultSettings: ScreenConfig = {
           enabled: false, interval: 60, output_dir: "", program: "", timestamp_format: ""
       };
       setSettings(defaultSettings);
       setTempInterval(defaultSettings.interval);
       setTempEnabled(defaultSettings.enabled);
       alert(`Failed to load settings: ${error}`);
    } finally {
      setIsLoadingSettings(false);
    }
  }

  async function saveSettings() {
    if (!settings) {
        alert("Cannot save settings: Current settings not loaded.");
        return;
    };

    const wasAutoRefreshEnabled = autoRefresh;
    if (wasAutoRefreshEnabled) {
      setAutoRefresh(false);
      if (refreshIntervalRef.current) {
        clearInterval(refreshIntervalRef.current);
        refreshIntervalRef.current = undefined;
      }
    }

    setIsSavingSettings(true);
    try {
        const updatedConfig: ScreenConfig = {
          ...settings,
          enabled: tempEnabled,
          interval: tempInterval,
          output_dir: settings.output_dir || "",
          program: settings.program || "",
          timestamp_format: settings.timestamp_format || ""
        };

        console.log("[Screen] Saving config via Tauri:", updatedConfig);

        await invoke("set_component_config", {
            componentName: "screen",
            config: updatedConfig
        });

        console.log("[Screen] Settings saved successfully via Tauri.");

        setSettings(updatedConfig);
        setShowSettings(false);

        if (wasAutoRefreshEnabled) {
          setTimeout(() => {
            setAutoRefresh(true);
          }, 1000); // Delay to ensure settings take effect
        }

        if (wasAutoRefreshEnabled && updatedConfig.enabled && updatedConfig.interval > 0) {
          console.log(`[Screen] Updated auto-refresh interval: ${updatedConfig.interval} seconds`);
        }

    } catch (error) {
      console.error('[Screen] Failed to save settings via Tauri:', error);
        alert(`Failed to save settings: ${error}`);
    } finally {
        setIsSavingSettings(false);
    }
  }

   function formatTimestamp(timestamp: number): string {
    const date = new Date(timestamp * 1000);
    return date.toLocaleString(undefined, {
      year: 'numeric', month: 'short', day: 'numeric',
      hour: '2-digit', minute: '2-digit', second: '2-digit'
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
    console.log("[Screen] Opening screenshot:", screenshot);
    const imageUrl = screenshot.dataUrl || `${API_BASE_URL}/api/files/screen/${screenshot.path}`;
    setSelectedImage(imageUrl);
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
    if (seconds <= 0) return "Off";
    if (seconds < 60) return `${seconds} sec`;
    if (seconds === 60) return "1 min";
    if (seconds < 3600) {
        const minutes = Math.floor(seconds / 60);
        const remainingSeconds = seconds % 60;
        return remainingSeconds === 0 ? `${minutes} min` : `${minutes}m ${remainingSeconds}s`;
    }
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    return minutes === 0 ? `${hours} hr` : `${hours}h ${minutes}m`;
  }

  function toggleSortOrder() {
    setSortOrder(prev => (prev === 'asc' ? 'desc' : 'asc'));
  }

  const sortedScreenshots = [...screenshots].sort((a, b) => {
     return sortOrder === 'asc' ? a.timestamp - b.timestamp : b.timestamp - a.timestamp;
   });


  return (
    <div className="p-6 md:p-8 space-y-6">
        {/* Header */}
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
                    disabled={isLoadingSettings}
                >
                    {isLoadingSettings ? (
                         <RefreshCw className="w-4 h-4 animate-spin" />
                    ) : (
                        <Settings className="w-4 h-4" />
                    )}
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
                    {settings ? (
                        <div className="space-y-6">
                            {/* Enable/Disable Setting */}
                            <div className="flex items-center justify-between">
                                <div className="flex items-center gap-3">
                                    <div className={`p-2 rounded-lg ${tempEnabled ? 'bg-green-500/10' : 'bg-[#1C2233]'}`}>
                                         <Power className={`w-5 h-5 ${tempEnabled ? 'text-green-500' : 'text-[#9CA3AF]'}`} />
                                    </div>
                                    <div>
                                        <p className="font-medium text-[#F9FAFB]">Enable Screenshots</p>
                                        <p className="text-sm text-[#9CA3AF]">
                                            {tempEnabled ? 'Screenshots are being captured' : 'Screenshot capture is paused'}
                                        </p>
                                    </div>
                                </div>
                                <Switch
                                    checked={tempEnabled}
                                    onCheckedChange={setTempEnabled}
                                    className="data-[state=checked]:bg-[#4C8BF5] data-[state=unchecked]:bg-[#1C2233]"
                                />
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
                                        min={5} // Min 5 seconds
                                        max={600} // Max 10 minutes
                                        step={5}
                                        value={[tempInterval]}
                                        onValueChange={(values) => setTempInterval(values[0])}
                                        disabled={!tempEnabled}
                                        className="data-[disabled]:opacity-50"
                                    />
                                     <div className="flex justify-between text-xs text-[#9CA3AF] mt-2">
                                         <span>5s</span>
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
                                     disabled={isSavingSettings || isLoadingSettings}
                                 >
                                     {isSavingSettings ? (
                                         <>
                                             <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
                                             Saving...
                                         </>
                                     ) : 'Save Settings'}
                                 </Button>
                             </div>
                        </div>
                    ) : (
                         <div className="text-center py-8 text-[#9CA3AF]">Loading settings...</div>
                    )}
                </div>
            </div>
        )}

      {/* Screenshot Display Area */}
      <div className="card">
        <div className="p-6">
          {/* Controls: Refresh, Sort, Auto-refresh, Pagination */}
          <div className="flex flex-col sm:flex-row justify-between gap-4 mb-8">
             {/* Left Controls */}
             <div className="flex items-center gap-2 flex-wrap">
                 <Button onClick={loadScreenshots} disabled={isLoading}>
                     <RefreshCw className={`w-4 h-4 mr-2 ${isLoading ? 'animate-spin' : ''}`} />
                     Refresh
                 </Button>
                 <Button onClick={toggleSortOrder} variant="secondary">
                      <ArrowUpDown className="w-4 h-4 mr-2" />
                      {sortOrder === 'asc' ? 'Oldest First' : 'Newest First'}
                  </Button>
                 <div className={`flex items-center gap-2 px-3 py-2 rounded-lg ${autoRefresh ? 'bg-[#4C8BF5]/20 border border-[#4C8BF5]/40' : 'bg-[#1C2233]'}`}>
                      <input
                          type="checkbox"
                          id="auto-refresh"
                          checked={autoRefresh}
                          onChange={(e) => setAutoRefresh(e.target.checked)}
                          disabled={!settings || !settings.enabled}
                          className="h-4 w-4 text-[#4C8BF5] rounded focus:ring-2 focus:ring-[#4C8BF5]/20 bg-[#232B3D] border-[#2A3142] disabled:opacity-50"
                      />
                      <label htmlFor="auto-refresh" className={`text-sm font-medium ${!settings || !settings.enabled ? 'text-[#9CA3AF]' : 'text-[#F9FAFB]'}`}>
                          Auto-refresh ({settings?.interval ? formatIntervalDisplay(settings.interval) : 'Off'})
                      </label>
                     {autoRefresh && settings?.enabled && (
                          <RefreshCw className="w-3 h-3 text-[#4C8BF5] animate-spin ml-1" style={{ animationDuration: '2s' }} />
                      )}
                  </div>
             </div>
             {/* Right Controls (Pagination) */}
             <div className="flex items-center gap-2">
                  <Button onClick={handlePreviousPage} disabled={currentPage === 1 || isLoading} variant="secondary">
                      Previous
                  </Button>
                  <div className="px-3 py-1 bg-[#1C2233] rounded-lg text-sm text-[#F9FAFB]">
                      Page {currentPage} of {totalPages}
                  </div>
                  <Button onClick={handleNextPage} disabled={currentPage >= totalPages || isLoading} variant="secondary">
                     Next
                  </Button>
             </div>
          </div>

          {/* Screenshot Grid or Loading/Empty State */}
          {isLoading && !isLoadingSettings ? ( // Show loading only if not also loading settings initially
             <div className="text-center py-12 text-[#9CA3AF]">
                 <RefreshCw className="w-8 h-8 animate-spin mx-auto mb-3 text-[#4C8BF5]" />
                 Loading screenshots...
             </div>
          ) : (
             <>
               {sortedScreenshots.length === 0 && !isLoading ? (
                   <div className="flex flex-col items-center justify-center py-12 text-[#9CA3AF]">
                        <Camera className="w-12 h-12 mb-4" />
                         <p className="mb-2">No screenshots found</p>
                         <p className="text-sm max-w-md text-center">
                             {settings?.enabled
                                 ? "Screenshots might be capturing. Try refreshing in a moment."
                                 : "Screenshot capture is disabled. Enable it in Settings."}
                         </p>
                     </div>
                ) : (
                     <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                         {sortedScreenshots.map((screenshot) => {
                             return (
                                <div
                                    key={screenshot.path}
                                    className="card card-hover overflow-hidden cursor-pointer group"
                                    onClick={() => handleScreenshotClick(screenshot)}
                                >
                                    <div className="aspect-video bg-[#1C2233] relative overflow-hidden">
                                        {screenshot.dataUrl ? (
                                            <img
                                                src={screenshot.dataUrl}
                                                alt={`Screenshot from ${formatTimestamp(screenshot.timestamp)}`}
                                                className="w-full h-full object-cover transition-transform duration-300 group-hover:scale-105"
                                                loading="lazy"
                                            />
                                        ) : (
                                             <div className="w-full h-full flex items-center justify-center text-[#9CA3AF]">
                                                 <RefreshCw className="w-6 h-6 animate-spin" />
                                             </div>
                                         )}
                                        <div className="absolute inset-0 bg-gradient-to-t from-black/70 via-black/30 to-transparent opacity-0 group-hover:opacity-100 transition-opacity flex items-end p-2">
                                             <p className="text-white text-xs truncate">Click to view full size</p>
                                         </div>
                                     </div>
                                     <div className="p-3">
                                         <div className="text-sm font-medium text-[#F9FAFB]">
                                             {formatTimestamp(screenshot.timestamp)}
                                         </div>
                                         <div className="text-xs text-gray-400 mt-1 truncate" title={screenshot.path}>
                                            {screenshot.path.split('/').pop()}
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
               className="fixed inset-0 bg-[#0F111A]/90 backdrop-blur-sm flex items-center justify-center z-50 p-4"
               onClick={closeModal}
           >
               <div
                   className="relative max-w-7xl max-h-[90vh] bg-[#1C2233] rounded-lg shadow-2xl overflow-hidden"
                   onClick={(e) => e.stopPropagation()}
               >
                    <img
                         src={selectedImage}
                         alt={`Full size screenshot from ${selectedScreenshot ? formatTimestamp(selectedScreenshot.timestamp) : ''}`}
                         className="block max-w-full max-h-[90vh] object-contain"
                     />
                   <Button
                       onClick={closeModal}
                       variant="secondary"
                       className="absolute top-2 right-2 rounded-full p-1.5 bg-[#1C2233]/70 hover:bg-[#232B3D]/90 backdrop-blur-sm z-10"
                   >
                       <X className="w-5 h-5 text-[#F9FAFB]" />
                   </Button>
                   {selectedScreenshot && (
                      <div className="absolute bottom-0 left-0 w-full p-2 bg-gradient-to-t from-black/80 to-transparent text-white text-xs">
                          {formatTimestamp(selectedScreenshot.timestamp)} - {selectedScreenshot.path}
                      </div>
                   )}
               </div>
           </div>
       )}
    </div>
  );
}