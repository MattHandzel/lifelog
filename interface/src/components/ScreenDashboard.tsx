import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core'; // Ensure invoke is imported
import { Button } from './ui/button';
import { Camera, X, Settings, Power, Clock, ArrowUpDown, RefreshCw } from 'lucide-react';
import { Slider } from './ui/slider';
import { Switch } from './ui/switch';


interface Screenshot {
  uuid: string;
  timestamp: number | null;
  dataUrl: string;    
  width: number;
  height: number;
  mime_type: string;
  origin_table: string; 
}

interface ScreenConfig {
	enabled: boolean;
	interval: number;
	output_dir: string;
	program: string;
	timestamp_format: string;
}

interface ScreenDashboardProps {
  collectorId: string | null;
}

interface LifelogDataKeyWrapper {
  uuid: string;
  origin: string;
}

export default function ScreenDashboard({ collectorId }: ScreenDashboardProps) {
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
  const [activeCollectorId, setActiveCollectorId] = useState<string | null>(collectorId);
  const pageSize = 9;
  const isLoadingRef = useRef(false);
  const [fetchError, setFetchError] = useState<string | null>(null); // For displaying fetch errors

  useEffect(() => {
    const initializeCollectorId = async () => {
      let resolvedCollectorId = collectorId;

      if (!collectorId) {
        try {
          const collectorIds = await invoke<string[]>('get_collector_ids');
          resolvedCollectorId = collectorIds.length > 0 ? collectorIds[0] : null;
        } catch (error) {
          console.error('[ScreenDashboard] Failed to fetch collector IDs:', error);
          resolvedCollectorId = null;
        }
      }

      setActiveCollectorId(resolvedCollectorId);

      if (resolvedCollectorId) {
        console.log(`[ScreenDashboard] Initializing with collectorId: ${resolvedCollectorId}`);
        setIsLoading(true);
        setIsLoadingSettings(true);
        loadSettings(resolvedCollectorId);
        loadScreenshots(resolvedCollectorId);
      }

      const savedAutoRefresh = localStorage.getItem('screenshots_auto_refresh');
      if (savedAutoRefresh !== null) {
        setAutoRefresh(savedAutoRefresh === 'true');
      }
    };

    initializeCollectorId();
  }, [collectorId]);

  useEffect(() => {
    if (activeCollectorId) {
      loadScreenshots(activeCollectorId);
    }
  }, [currentPage, sortOrder, activeCollectorId]);

  useEffect(() => {
    if (refreshIntervalRef.current) {
      clearInterval(refreshIntervalRef.current);
      refreshIntervalRef.current = undefined;
    }
    if (autoRefresh && settings && settings.enabled && settings.interval > 0 && activeCollectorId) {
       refreshIntervalRef.current = window.setInterval(() => {
         if (currentPage === 1 && sortOrder === 'desc') {
           console.log('[ScreenDashboard] Auto-refresh triggered...');
           loadScreenshots(activeCollectorId);
         }
       }, settings.interval * 1000);
    }
    localStorage.setItem('screenshots_auto_refresh', autoRefresh.toString());
    return () => { if (refreshIntervalRef.current) clearInterval(refreshIntervalRef.current); };
  }, [autoRefresh, settings, currentPage, sortOrder, activeCollectorId]);

  async function loadScreenshots(collectorIdToLoad: string | null) {
    if (isLoadingRef.current) {
      console.log("[ScreenDashboard] loadScreenshots: Load already in progress, skipping.");
      return;
    }
    if (!collectorIdToLoad) {
      console.log("[ScreenDashboard] loadScreenshots: No collectorId provided, cannot load.");
      setScreenshots([]);
      setIsLoading(false);
      setTotalPages(1);
      setCurrentPage(1);
      setFetchError(null)
      return;
    }

    isLoadingRef.current = true;
    console.log(`[ScreenDashboard] loadScreenshots: STARTING load for collector ${collectorIdToLoad}`);
    setIsLoading(true);
    setFetchError(null);
    const startTime = performance.now();
    try {
      console.log(`[ScreenDashboard] loadScreenshots: Step 1 - Invoking query_screenshot_keys for ${collectorIdToLoad}`);
      const keys = await invoke<LifelogDataKeyWrapper[]>("query_screenshot_keys", { collectorId: collectorIdToLoad });
      const postQueryKeysTime = performance.now();
      console.log(`[ScreenDashboard] loadScreenshots: Step 1 - Received ${keys.length} raw keys for ${collectorIdToLoad}. Time: ${(postQueryKeysTime - startTime).toFixed(2)}ms`, keys.slice(0, 5)); // Log first 5 keys

      if (keys.length === 0) {
        setScreenshots([]);
        setTotalPages(1);
        setCurrentPage(1);
        setIsLoading(false);
        console.log(`[ScreenDashboard] loadScreenshots: No raw keys found. Total time: ${(performance.now() - startTime).toFixed(2)}ms`);
        return;
      }

      const screenKeys = keys.filter(key => {
        const parts = key.origin.split(':');
        return parts.length > 0 && parts[parts.length - 1].toLowerCase() === 'screen';
      });
      const postFilterTime = performance.now();
      console.log(`[ScreenDashboard] loadScreenshots: Step 2 - Filtered down to ${screenKeys.length} screen-specific keys. Time since start: ${(postFilterTime - startTime).toFixed(2)}ms. First 5 screen keys:`, screenKeys.slice(0,5));

      if (screenKeys.length === 0) {
        setScreenshots([]);
        setTotalPages(1);
        setCurrentPage(1);
        setIsLoading(false);
        console.log(`[ScreenDashboard] loadScreenshots: No screen-specific keys after filtering. Total time: ${(performance.now() - startTime).toFixed(2)}ms`);
        return;
      }

      const LATEST_LIMIT = 10;
      let keysToFetch = screenKeys;
      if (screenKeys.length > LATEST_LIMIT) {
        keysToFetch = screenKeys.slice(0, LATEST_LIMIT); 
        console.log(`[ScreenDashboard] loadScreenshots: Sliced to ${keysToFetch.length} keys due to LATEST_LIMIT of ${LATEST_LIMIT}.`);
      }

      console.log(`[ScreenDashboard] loadScreenshots: PRE-STEP 3 - keysToFetch has ${keysToFetch.length} items. Content (first 5):`, JSON.stringify(keysToFetch.slice(0,5)));

      if (keysToFetch.length === 0) {
        console.log("[ScreenDashboard] loadScreenshots: keysToFetch is empty right before invoking get_screenshots_data. Returning.");
        setScreenshots([]);
        setTotalPages(1);
        setCurrentPage(1);
        setIsLoading(false);
        return;
      }

      console.log(`[ScreenDashboard] loadScreenshots: Step 3 - Invoking get_screenshots_data for ${keysToFetch.length} screen keys.`);
      const startTimeStep3 = performance.now();
      try {
        const fetchedScreenshots = await invoke<Screenshot[]>("get_screenshots_data", { keys: keysToFetch });
        const endTimeStep3 = performance.now();
        console.log(`[ScreenDashboard] loadScreenshots: Step 3 - SUCCESS - Received ${fetchedScreenshots.length} screenshot data items. Time since start: ${(endTimeStep3 - startTimeStep3).toFixed(2)}ms. First item:`, fetchedScreenshots.length > 0 ? fetchedScreenshots[0] : 'No data');
        setScreenshots(fetchedScreenshots);

        setTotalPages(Math.ceil(fetchedScreenshots.length / pageSize)); 
        if (currentPage > Math.ceil(fetchedScreenshots.length / pageSize) && fetchedScreenshots.length > 0) {
            setCurrentPage(1);
        } else if (fetchedScreenshots.length === 0) {
            setCurrentPage(1);
        }
      } catch (error) {
        const endTimeStep3Error = performance.now();
        console.error(`[ScreenDashboard] loadScreenshots: Step 3 - FAILED - Error invoking get_screenshots_data:`, error);
        console.error(`[ScreenDashboard] loadScreenshots: Step 3 - Time for invoke attempt: ${(endTimeStep3Error - startTimeStep3).toFixed(2)}ms. Time since start: ${(endTimeStep3Error - startTime).toFixed(2)}ms.`);
        setScreenshots([]);
        setTotalPages(1);
        setCurrentPage(1);
        const errorMessage = error instanceof Error ? error.message :
                             typeof error === 'string' ? error :
                             typeof error === 'object' && error !== null && 'message' in error && typeof error.message === 'string' ? error.message :
                             "Failed to load screenshots.";
        setFetchError(errorMessage);
      }

    } catch (error: any) {
      console.error('[ScreenDashboard] Failed to load screenshots via Tauri (outer catch):', error);
      setScreenshots([]);
      setTotalPages(1);
      setCurrentPage(1);
      const errorMessage = error instanceof Error ? error.message :
                           typeof error === 'string' ? error :
                           typeof error === 'object' && error !== null && 'message' in error && typeof error.message === 'string' ? error.message :
                           "Failed to load screenshots.";
      setFetchError(errorMessage);
    } finally {
      setIsLoading(false);
      isLoadingRef.current = false; // Clear loading flag
      console.log(`[ScreenDashboard] loadScreenshots: FINISHED for collector ${collectorIdToLoad}. Total time: ${(performance.now() - startTime).toFixed(2)}ms`);
    }
  }

  async function loadSettings(currentCollectorIdToLoad: string, retries = 3, delay = 1000) {
    setIsLoadingSettings(true);
    try {
      console.log(`[ScreenDashboard] Requesting screen config for collector ${currentCollectorIdToLoad} via Tauri... (Attempt: ${4 - retries})`);
      const result = await invoke("get_component_config", {
        collectorId: currentCollectorIdToLoad,
        componentType: "screen"
      });
      console.log(`[ScreenDashboard] Received screen config for collector ${currentCollectorIdToLoad} from Tauri:`, result);
      if (result && typeof result === 'object') {
         const loadedSettings = result as ScreenConfig;
         if (typeof loadedSettings.enabled !== 'boolean' || typeof loadedSettings.interval !== 'number') {
            console.error("[ScreenDashboard] Received invalid settings format from backend.");
            throw new Error("Received invalid settings format from backend.");
         }
        setSettings(loadedSettings);
        setTempInterval(loadedSettings.interval);
        setTempEnabled(loadedSettings.enabled);
        setIsLoadingSettings(false); // Success
        return;
      } else {
          console.warn("[ScreenDashboard] Backend returned null or unexpected for screen config. Using defaults for now. This might indicate component not fully configured on server.");
      }
    } catch (error: any) {
      console.error(`[ScreenDashboard] Failed to load screen settings for collector ${currentCollectorIdToLoad} via Tauri (Attempt: ${4 - retries}):`, error);
      if (retries > 0 && error.message && error.message.includes("No collector found")) {
        console.log(`[ScreenDashboard] Collector not found, retrying loadSettings in ${delay / 1000}s... (${retries} retries left)`);
        await new Promise(resolve => setTimeout(resolve, delay));
        if (currentCollectorIdToLoad) {
          await loadSettings(currentCollectorIdToLoad, retries - 1, delay * 2);
        }
        return;
      }
       alert(`Failed to load settings for ${currentCollectorIdToLoad} after multiple attempts: ${error.message || error}`);
    }

    console.warn(`[ScreenDashboard] Falling back to default settings for collector ${currentCollectorIdToLoad} after attempts or non-retryable error.`);
    const defaultSettingsData: ScreenConfig = {
        enabled: false, interval: 60, output_dir: "", program: "", timestamp_format: ""
    };
    setSettings(defaultSettingsData);
    setTempInterval(defaultSettingsData.interval);
    setTempEnabled(defaultSettingsData.enabled);
    setIsLoadingSettings(false);
  }

  async function saveSettings() {
    if (!settings || !activeCollectorId) {
        alert("Cannot save settings: Current settings not loaded or collector ID unknown.");
        return;
    };
    const wasAutoRefreshEnabled = autoRefresh;
    if (wasAutoRefreshEnabled) {
      setAutoRefresh(false);
      if (refreshIntervalRef.current) clearInterval(refreshIntervalRef.current);
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
        console.log(`[ScreenDashboard] Saving screen config for collector ${activeCollectorId} via Tauri:`, updatedConfig);
        await invoke("set_component_config", {
            collectorId: activeCollectorId,
            componentType: "screen",
            configValue: updatedConfig
        });
        console.log(`[ScreenDashboard] Screen settings for collector ${activeCollectorId} saved successfully.`);
        setSettings(updatedConfig);
        setShowSettings(false);
    } catch (error) {
        console.error(`[ScreenDashboard] Failed to save settings for collector ${activeCollectorId} via Tauri:`, error);
        alert(`Failed to save settings for ${activeCollectorId}: ${error}`);
    } finally {
        setIsSavingSettings(false);
        if (wasAutoRefreshEnabled) {
           setTimeout(() => setAutoRefresh(true), 500);
        }
    }
  }

   function formatTimestamp(timestamp: number | null): string {
    if (timestamp === null) return "N/A";
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
    console.log("Opening screenshot:", screenshot);
    setSelectedImage(screenshot.dataUrl); 
    setSelectedScreenshot(screenshot);
  }

  function closeModal() {
    setSelectedImage(null);
    setSelectedScreenshot(null);
  }

  function toggleSettings() {
    console.log('[ScreenDashboard] Toggling settings panel. Current activeCollectorId:', activeCollectorId);
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
    const tsA = a.timestamp === null ? (sortOrder === 'asc' ? Infinity : -Infinity) : a.timestamp;
    const tsB = b.timestamp === null ? (sortOrder === 'asc' ? Infinity : -Infinity) : b.timestamp;
    return sortOrder === 'asc' ? tsA - tsB : tsB - tsA;
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
                    disabled={isLoadingSettings} // Disable while loading initial settings
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
                                        min={5} // Example: Minimum 5 seconds
                                        max={600} // Example: Maximum 10 minutes
                                        step={5}
                                        value={[tempInterval]}
                                        onValueChange={(values) => setTempInterval(values[0])}
                                        disabled={!tempEnabled} // Disable slider if capture is off
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

      <div className="card">
        <div className="p-6">
          <div className="flex flex-col sm:flex-row justify-between gap-4 mb-8">
             <div className="flex items-center gap-2 flex-wrap">
                 <Button onClick={() => {
                     if (activeCollectorId) {
                       loadScreenshots(activeCollectorId);
                     }
                 }}
                     disabled={isLoading || !activeCollectorId}
                 >
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
                          disabled={!settings || !settings.enabled} // Disable if logger off
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

          {isLoading && !isLoadingSettings ? (
             <div className="text-center py-12 text-[#9CA3AF]">
                 <RefreshCw className="w-8 h-8 animate-spin mx-auto mb-3 text-[#4C8BF5]" />
                 Loading screenshots...
             </div>
          ) : (
             <>
               {fetchError && (
                <div className="mb-4 p-4 text-center text-red-400 bg-red-900/30 border border-red-600/50 rounded-md">
                  <p>Error loading screenshots: {fetchError}</p>
                  <p className="text-sm text-red-400/80">The server might be busy or unresponsive. Some images may be missing.</p>
                </div>
              )}
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
                                    key={screenshot.uuid} // Use uuid as key
                                    className="card card-hover overflow-hidden cursor-pointer group"
                                    onClick={() => handleScreenshotClick(screenshot)}
                                >
                                    <div className="aspect-video bg-[#1C2233] relative overflow-hidden">
                                        {screenshot.dataUrl ? (
                                            <img
                                                src={screenshot.dataUrl}
                                                alt={`Screenshot from ${formatTimestamp(screenshot.timestamp)}`}
                                                className="w-full h-full object-cover transition-transform duration-300 group-hover:scale-105"
                                                loading="lazy" // Lazy load images
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
                                         <div className="text-xs text-gray-400 mt-1 truncate" title={screenshot.origin_table + '/' + screenshot.uuid}>
                                            {screenshot.uuid} {/* Show UUID or a derived name */}
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
                         src={selectedImage} // Use the stored URL (dataUrl)
                         alt={`Full size screenshot from ${selectedScreenshot && selectedScreenshot.timestamp ? formatTimestamp(selectedScreenshot.timestamp) : selectedScreenshot?.uuid || ''}`}
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
                          {(selectedScreenshot.timestamp ? formatTimestamp(selectedScreenshot.timestamp) : 'No Timestamp')} - ID: {selectedScreenshot.uuid}
                      </div>
                   )}
               </div>
           </div>
       )}
    </div>
  );
}
