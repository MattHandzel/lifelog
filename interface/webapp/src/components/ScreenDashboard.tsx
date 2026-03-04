import { useState, useEffect, useRef } from 'react';
import { Button } from './ui/button';
import { Camera, X, Settings, Power, Clock, ArrowUpDown, RefreshCw, ChevronLeft, ChevronRight, ArrowRight } from 'lucide-react';
import { Slider } from './ui/slider';
import { Switch } from './ui/switch';
import { client } from '../lib/client';
import { cn } from '../lib/utils';
import { 
  ListModalitiesRequest, 
  QueryRequest, 
  GetDataRequest, 
  Query, 
  LifelogDataKey,
  GetSystemConfigRequest,
  SetSystemConfigRequest
} from '../gen/lifelog_pb';
import { SystemConfig } from '../gen/lifelog_types_pb';

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

export default function ScreenDashboard({ collectorId }: ScreenDashboardProps): JSX.Element {
  const [screenshots, setScreenshots] = useState<Screenshot[]>([]);
  const [currentPage, setCurrentPage] = useState(1);
  const [isLoading, setIsLoading] = useState(false);
  const [selectedImage, setSelectedImage] = useState<string | null>(null);
  const [selectedScreenshot, setSelectedScreenshot] = useState<Screenshot | null>(null);
  const [totalPages, setTotalPages] = useState(1);
  const [showSettings, setShowSettings] = useState(false);
  const [settings, setSettings] = useState<ScreenConfig | null>(null);
  const [isSavingSettings, setIsSavingSettings] = useState(false);
  const [tempInterval, setTempInterval] = useState(60);
  const [tempEnabled, setTempEnabled] = useState(true);
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('desc');
  const [activeCollectorId, setActiveCollectorId] = useState<string | null>(collectorId);
  const pageSize = 9;
  const isLoadingRef = useRef(false);

  useEffect(function () {
    async function initializeCollectorId(): Promise<void> {
      let resolvedCollectorId = collectorId;
      if (!collectorId) {
        try {
          const resp = await client.listModalities(new ListModalitiesRequest());
          const ids = Array.from(new Set(resp.modalities.map(m => m.streamId.split(':')[0])));
          resolvedCollectorId = ids.length > 0 ? ids[0] : null;
        } catch (error) {
          console.error('[ScreenDashboard] Failed to fetch collector IDs:', error);
        }
      }
      setActiveCollectorId(resolvedCollectorId);
      if (resolvedCollectorId) {
        loadSettings(resolvedCollectorId);
        loadScreenshots(resolvedCollectorId);
      }
    }
    initializeCollectorId();
  }, [collectorId]);

  useEffect(function () {
    if (activeCollectorId) {
      loadScreenshots(activeCollectorId);
    }
  }, [currentPage, sortOrder, activeCollectorId]);

  async function loadScreenshots(collectorIdToLoad: string | null): Promise<void> {
    if (isLoadingRef.current || !collectorIdToLoad) return;
    isLoadingRef.current = true;
    setIsLoading(true);
    try {
      const query = new Query({
        searchOrigins: [`${collectorIdToLoad}:screen`],
        returnOrigins: [`${collectorIdToLoad}:screen`],
      });
      const resp = await client.query(new QueryRequest({ query }));
      const keys = resp.keys;
      if (keys.length === 0) {
        setScreenshots([]);
        setTotalPages(1);
        return;
      }
      const start = (currentPage - 1) * pageSize;
      const end = start + pageSize;
      const keysToFetch = keys.slice(start, end).map(k => new LifelogDataKey({ uuid: k.uuid, origin: k.origin }));
      const dataResp = await client.getData(new GetDataRequest({ keys: keysToFetch }));
      const fetchedScreenshots: Screenshot[] = dataResp.data.map(d => {
        if (d.payload.case === 'screenframe') {
          const f = d.payload.value;
          return {
            uuid: f.uuid,
            timestamp: Number(f.timestamp?.seconds || 0),
            dataUrl: f.mediaUrl || f.dataUrl,
            width: f.width,
            height: f.height,
            mime_type: f.mimeType,
            origin_table: `${collectorIdToLoad}:screen`
          };
        }
        return null;
      }).filter((s): s is Screenshot => s !== null);
      setScreenshots(fetchedScreenshots);
      setTotalPages(Math.ceil(keys.length / pageSize));
    } catch (error: any) {
      console.error('[ScreenDashboard] Failed to load screenshots:', error);
    } finally {
      setIsLoading(false);
      isLoadingRef.current = false;
    }
  }

  async function loadSettings(currentCollectorIdToLoad: string): Promise<void> {
    try {
      const resp = await client.getConfig(new GetSystemConfigRequest());
      const config = resp.config;
      if (config && config.collectors[currentCollectorIdToLoad]) {
        const screen = config.collectors[currentCollectorIdToLoad].screen;
        if (screen) {
          setSettings({
            enabled: screen.enabled,
            interval: screen.interval,
            output_dir: screen.outputDir,
            program: screen.program,
            timestamp_format: screen.timestampFormat
          });
          setTempInterval(screen.interval);
          setTempEnabled(screen.enabled);
        }
      }
    } catch (error: any) {
      console.error(`[ScreenDashboard] Failed to load settings:`, error);
    }
  }

  async function saveSettings(): Promise<void> {
    if (!settings || !activeCollectorId) return;
    setIsSavingSettings(true);
    try {
        const resp = await client.getConfig(new GetSystemConfigRequest());
        const config = resp.config || new SystemConfig();
        if (config.collectors[activeCollectorId]) {
          const screen = config.collectors[activeCollectorId].screen;
          if (screen) {
            screen.enabled = tempEnabled;
            screen.interval = tempInterval;
          }
        }
        await client.setConfig(new SetSystemConfigRequest({ config }));
        setSettings({ ...settings, enabled: tempEnabled, interval: tempInterval });
        setShowSettings(false);
    } catch (error: any) {
        console.error(`[ScreenDashboard] Failed to save settings:`, error);
    } finally {
        setIsSavingSettings(false);
    }
  }

  function formatTimestamp(timestamp: number | null): string {
    if (!timestamp) return "N/A";
    return new Date(timestamp * 1000).toLocaleString();
  }

  const sortedScreenshots = [...screenshots].sort((a, b) => {
    const tsA = a.timestamp || 0;
    const tsB = b.timestamp || 0;
    return sortOrder === 'asc' ? tsA - tsB : tsB - tsA;
  });

  return (
    <div className="p-6 md:p-8 space-y-6 bg-slate-950 min-h-full text-slate-100 font-sans">
        <div className="flex items-center justify-between">
            <div>
                <div className="flex items-center gap-3">
                    <Camera className="w-8 h-8 text-blue-500" />
                    <h1 className="text-2xl font-bold tracking-tight">Screenshots</h1>
                </div>
                <p className="text-slate-400">Viewing capture stream from {activeCollectorId}</p>
            </div>
            <Button onClick={() => setShowSettings(!showSettings)} variant="outline" className="border-slate-800 bg-slate-900/50 hover:bg-slate-800">
                <Settings className="w-4 h-4 mr-2" /> Settings
            </Button>
        </div>

        {showSettings && (
            <div className="bg-slate-900 rounded-2xl p-6 border border-slate-800 shadow-2xl animate-in fade-in zoom-in duration-200">
                <h2 className="text-lg font-semibold mb-6 flex items-center gap-2">
                  <Settings className="w-5 h-5 text-blue-500" /> Capture Configuration
                </h2>
                <div className="space-y-8 max-w-md">
                    <div className="flex items-center justify-between p-4 bg-slate-950 rounded-xl border border-slate-800/50">
                        <div className="flex items-center gap-3">
                            <div className={cn("p-2 rounded-lg", tempEnabled ? "bg-green-500/10" : "bg-slate-800")}>
                              <Power className={cn("w-5 h-5", tempEnabled ? "text-green-500" : "text-slate-500")} />
                            </div>
                            <span className="font-medium">Active Capture</span>
                        </div>
                        <Switch checked={tempEnabled} onCheckedChange={setTempEnabled} />
                    </div>
                    <div className="space-y-4 px-2">
                        <div className="flex justify-between text-sm font-medium">
                            <span className="text-slate-400">Capture Interval</span>
                            <span className="text-blue-400 font-mono">{tempInterval}s</span>
                        </div>
                        <Slider min={5} max={600} step={5} value={[tempInterval]} onValueChange={(v) => setTempInterval(v[0])} />
                    </div>
                    <div className="flex justify-end gap-3 pt-4">
                        <Button variant="ghost" onClick={() => setShowSettings(false)}>Cancel</Button>
                        <Button onClick={saveSettings} disabled={isSavingSettings} className="bg-blue-600 hover:bg-blue-700">
                            {isSavingSettings ? 'Applying...' : 'Save Settings'}
                        </Button>
                    </div>
                </div>
            </div>
        )}

        <div className="bg-slate-900/50 rounded-2xl border border-slate-800 overflow-hidden shadow-2xl">
          <div className="p-4 border-b border-slate-800 flex flex-wrap gap-4 justify-between items-center bg-slate-900">
             <div className="flex items-center gap-2">
                 <Button size="sm" onClick={() => activeCollectorId && loadScreenshots(activeCollectorId)} disabled={isLoading} className="bg-slate-800 border-slate-700 hover:bg-slate-700">
                     <RefreshCw className={cn("w-4 h-4 mr-2", isLoading && "animate-spin")} /> Refresh
                 </Button>
                 <Button size="sm" variant="outline" onClick={() => setSortOrder(sortOrder === 'asc' ? 'desc' : 'asc')} className="border-slate-800">
                      <ArrowUpDown className="w-4 h-4 mr-2" /> {sortOrder === 'asc' ? 'Oldest' : 'Newest'}
                  </Button>
             </div>
             <div className="flex items-center gap-4">
                  <div className="flex items-center gap-1">
                    <Button size="icon" variant="ghost" onClick={() => setCurrentPage(Math.max(1, currentPage - 1))} disabled={currentPage === 1} className="h-8 w-8 text-slate-400 hover:text-white">
                        <ChevronLeft className="w-4 h-4" />
                    </Button>
                    <span className="text-xs font-bold text-slate-500 uppercase tracking-tighter w-20 text-center">Page {currentPage} / {totalPages}</span>
                    <Button size="icon" variant="ghost" onClick={() => setCurrentPage(Math.min(totalPages, currentPage + 1))} disabled={currentPage === totalPages} className="h-8 w-8 text-slate-400 hover:text-white">
                       <ChevronRight className="w-4 h-4" />
                    </Button>
                  </div>
             </div>
          </div>

          <div className="p-6">
            {isLoading ? (
               <div className="py-24 text-center">
                   <div className="relative w-16 h-16 mx-auto mb-6">
                     <div className="absolute inset-0 border-4 border-blue-500/20 rounded-full animate-spin"></div>
                   </div>
                   <p className="text-slate-400 font-medium">Syncing with remote vault...</p>
               </div>
            ) : sortedScreenshots.length === 0 ? (
                <div className="py-24 text-center text-slate-500 flex flex-col items-center">
                    <Camera className="w-10 h-10 opacity-20 mb-6" />
                    <p className="text-lg font-semibold text-slate-300">No Captures Found</p>
                </div>
            ) : (
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
                    {sortedScreenshots.map((s) => (
                        <div key={s.uuid} className="group relative bg-slate-900 rounded-2xl overflow-hidden border border-slate-800 hover:border-blue-500/50 hover:shadow-2xl hover:shadow-blue-500/5 transition-all duration-300 cursor-pointer" onClick={() => {setSelectedImage(s.dataUrl); setSelectedScreenshot(s);}}>
                            <div className="aspect-video relative overflow-hidden bg-slate-950">
                                <img src={s.dataUrl} alt="" className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-700 ease-out" loading="lazy" />
                            </div>
                            <div className="p-4 flex items-center justify-between">
                                <div className="min-w-0">
                                  <p className="text-sm font-semibold truncate text-slate-200">{formatTimestamp(s.timestamp)}</p>
                                  <p className="text-[10px] text-slate-500 font-mono truncate tracking-tight">{s.uuid}</p>
                                </div>
                                <ArrowRight className="w-4 h-4 text-blue-400 opacity-0 group-hover:opacity-100 transition-opacity" />
                            </div>
                        </div>
                    ))}
                </div>
            )}
          </div>
        </div>

        {selectedImage && (
           <div className="fixed inset-0 bg-slate-950/95 backdrop-blur-xl flex items-center justify-center z-50 p-4 sm:p-8 animate-in fade-in duration-300" onClick={() => setSelectedImage(null)}>
               <div className="relative w-full h-full flex flex-col items-center justify-center" onClick={e => e.stopPropagation()}>
                    <div className="relative group max-w-full max-h-full">
                      <img src={selectedImage} alt="" className="max-w-full max-h-[85vh] rounded-2xl shadow-2xl border border-slate-800" />
                      <Button variant="outline" size="icon" className="absolute -top-4 -right-4 rounded-full bg-slate-900 border-slate-700 shadow-xl" onClick={() => setSelectedImage(null)}>
                          <X className="w-5 h-5" />
                      </Button>
                    </div>
                    {selectedScreenshot && (
                        <div className="mt-8 bg-slate-900/80 p-6 rounded-3xl backdrop-blur-xl border border-slate-800 shadow-2xl max-w-2xl w-full flex items-center justify-between">
                            <div className="space-y-1">
                              <p className="text-xl font-bold text-white">{formatTimestamp(selectedScreenshot.timestamp)}</p>
                              <p className="text-xs text-slate-500 font-mono">{selectedScreenshot.uuid}</p>
                            </div>
                            <div className="flex gap-3">
                              <Button variant="outline" size="sm" className="border-slate-800">Export</Button>
                              <Button size="sm" className="bg-blue-600 hover:bg-blue-700">Thread</Button>
                            </div>
                        </div>
                    )}
               </div>
           </div>
        )}
    </div>
  );
}
