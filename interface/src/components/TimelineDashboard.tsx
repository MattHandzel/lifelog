import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Clock, Search, RefreshCw } from 'lucide-react';
import { Button } from './ui/button';
import { Input } from './ui/input';
import ResultCard, { FrameDataWrapper } from './ResultCard';

interface TimelineEntry {
  uuid: string;
  origin: string;
  modality: string;
  timestamp: number | null;
}

interface TimelineDashboardProps {
  collectorId?: string | null;
}

export default function TimelineDashboard({ collectorId = null }: TimelineDashboardProps): JSX.Element {
  const [startDate, setStartDate] = useState<string>(() => {
    const d = new Date();
    d.setHours(0, 0, 0, 0);
    return d.toISOString().slice(0, 16);
  });
  const [endDate, setEndDate] = useState<string>(() => {
    const d = new Date();
    d.setHours(23, 59, 59, 0);
    return d.toISOString().slice(0, 16);
  });
  const [textQuery, setTextQuery] = useState<string>('');
  const [queryMode, setQueryMode] = useState<'text' | 'llql'>('text');
  const [results, setResults] = useState<TimelineEntry[]>([]);
  const [frames, setFrames] = useState<FrameDataWrapper[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  async function handleSearch(): Promise<void> {
    setIsLoading(true);
    try {
      const startTime = startDate ? Math.floor(new Date(startDate).getTime() / 1000) : undefined;
      const endTime = endDate ? Math.floor(new Date(endDate).getTime() / 1000) : undefined;
      const q = textQuery.trim();
      const queryText = q.length === 0
        ? undefined
        : queryMode === 'llql'
          ? [(q.startsWith('llql:') || q.startsWith('llql-json:')) ? q : `llql:${q}`]
          : [q];

      const entries = await invoke<TimelineEntry[]>('query_timeline', {
        collectorId: collectorId || undefined,
        textQuery: queryText,
        startTime,
        endTime,
      });

      setResults(entries);

      if (entries.length > 0) {
        const keys = entries.slice(0, 50).map(e => ({ uuid: e.uuid, origin: e.origin }));
        const enriched = await invoke<FrameDataWrapper[]>('get_frame_data', { keys });
        const frameMap = new Map(enriched.map(f => [f.uuid, f]));
        const ordered = entries.slice(0, 50).map(e => frameMap.get(e.uuid) ?? {
          uuid: e.uuid, modality: e.modality, timestamp: e.timestamp,
          text: null, url: null, title: null, visit_count: null,
          command: null, working_dir: null, exit_code: null,
          application: null, window_title: null, duration_secs: null,
          audio_data_url: null, codec: null, sample_rate: null, channels: null, audio_duration_secs: null,
          image_data_url: null, width: null, height: null, mime_type: null,
          camera_device: null, processes: null,
          transcription_model: null, transcription_confidence: null, source_frame_uuid: null,
        });
        setFrames(ordered);
      } else {
        setFrames([]);
      }
    } catch (error) {
      console.error('Timeline query failed:', error);
      setResults([]);
      setFrames([]);
    } finally {
      setIsLoading(false);
    }
  }

  useEffect(() => { handleSearch(); }, []);

  function handleReset(): void {
    setStartDate('');
    setEndDate('');
    setTextQuery('');
    setResults([]);
    setFrames([]);
  }

  return (
    <div className="p-6 md:p-8 space-y-6">
      <div className="mb-8">
        <div className="flex items-center gap-3 mb-2">
          <Clock className="w-8 h-8 text-[#4C8BF5]" />
          <h1 className="title">Timeline</h1>
        </div>
        <p className="subtitle">Browse your lifelog events by time and search</p>
      </div>

      {/* Search Controls */}
      <div className="card mb-6">
        <div className="p-6 space-y-4">
          {/* Text Search */}
          <div className="space-y-2">
            <div className="flex items-center justify-between gap-2">
              <label className="block text-sm font-medium text-[#F9FAFB]">Search</label>
              <div className="flex gap-2">
                <Button
                  type="button"
                  size="sm"
                  variant={queryMode === 'text' ? 'secondary' : 'outline'}
                  onClick={() => setQueryMode('text')}
                  disabled={isLoading}
                >
                  Text
                </Button>
                <Button
                  type="button"
                  size="sm"
                  variant={queryMode === 'llql' ? 'secondary' : 'outline'}
                  onClick={() => setQueryMode('llql')}
                  disabled={isLoading}
                >
                  LLQL
                </Button>
              </div>
            </div>
            <div className="relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-[#9CA3AF] w-5 h-5" />
              <Input
                type="text"
                placeholder={queryMode === 'llql' ? 'Enter LLQL JSON (or llql:...)' : 'Enter search query...'}
                value={textQuery}
                onChange={(e) => setTextQuery(e.target.value)}
                className="pl-10 bg-[#1C2233] border-[#232B3D] text-[#F9FAFB]"
              />
            </div>
          </div>

          {/* Date Range */}
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <label className="block text-sm font-medium text-[#F9FAFB]">Start Date</label>
              <Input
                type="datetime-local"
                value={startDate}
                onChange={(e) => setStartDate(e.target.value)}
                className="bg-[#1C2233] border-[#232B3D] text-[#F9FAFB]"
              />
            </div>
            <div className="space-y-2">
              <label className="block text-sm font-medium text-[#F9FAFB]">End Date</label>
              <Input
                type="datetime-local"
                value={endDate}
                onChange={(e) => setEndDate(e.target.value)}
                className="bg-[#1C2233] border-[#232B3D] text-[#F9FAFB]"
              />
            </div>
          </div>

          {/* Action Buttons */}
          <div className="flex gap-2 justify-end border-t border-[#232B3D] pt-4">
            <Button
              onClick={handleReset}
              variant="secondary"
              disabled={isLoading}
            >
              Reset
            </Button>
            <Button
              onClick={handleSearch}
              disabled={isLoading}
            >
              {isLoading ? (
                <>
                  <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
                  Searching...
                </>
              ) : (
                <>
                  <Search className="w-4 h-4 mr-2" />
                  Search
                </>
              )}
            </Button>
          </div>
        </div>
      </div>

      {/* Results */}
      <div className="card">
        <div className="p-6">
          {isLoading ? (
            <div className="flex flex-col items-center justify-center py-12 text-[#9CA3AF]">
              <RefreshCw className="w-8 h-8 animate-spin mb-3 text-[#4C8BF5]" />
              Searching timeline...
            </div>
          ) : results.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-12 text-center text-[#9CA3AF]">
              <Clock className="w-12 h-12 mb-4" />
              <p className="text-lg font-medium text-[#F9FAFB] mb-2">No results found</p>
              <p>Try adjusting your search criteria</p>
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
              {(frames.length > 0 ? frames : results.map(e => ({
                uuid: e.uuid, modality: e.modality, timestamp: e.timestamp,
                text: null, url: null, title: null, visit_count: null,
                command: null, working_dir: null, exit_code: null,
                application: null, window_title: null, duration_secs: null,
                audio_data_url: null, codec: null, sample_rate: null, channels: null, audio_duration_secs: null,
                image_data_url: null, width: null, height: null, mime_type: null,
                camera_device: null, processes: null,
          transcription_model: null, transcription_confidence: null, source_frame_uuid: null,
              }))).map((frame) => (
                <ResultCard key={frame.uuid} frame={frame} />
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
