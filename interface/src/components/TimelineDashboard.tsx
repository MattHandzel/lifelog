import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Clock, Camera, Mic, FileText, Search, RefreshCw } from 'lucide-react';
import { Button } from './ui/button';
import { Input } from './ui/input';

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
  const [startDate, setStartDate] = useState<string>('');
  const [endDate, setEndDate] = useState<string>('');
  const [textQuery, setTextQuery] = useState<string>('');
  const [results, setResults] = useState<TimelineEntry[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  function getModalityIcon(modality: string): JSX.Element {
    switch (modality?.toLowerCase()) {
      case 'screen':
        return <Camera className="w-4 h-4 text-blue-400" />;
      case 'audio':
        return <Mic className="w-4 h-4 text-green-400" />;
      case 'camera':
        return <Camera className="w-4 h-4 text-purple-400" />;
      default:
        return <FileText className="w-4 h-4 text-gray-400" />;
    }
  }

  function formatTimestamp(timestamp: number | null): string {
    if (!timestamp) return 'N/A';
    return new Date(timestamp * 1000).toLocaleString();
  }

  async function handleSearch(): Promise<void> {
    setIsLoading(true);
    try {
      const startTime = startDate ? Math.floor(new Date(startDate).getTime() / 1000) : undefined;
      const endTime = endDate ? Math.floor(new Date(endDate).getTime() / 1000) : undefined;

      const entries = await invoke<TimelineEntry[]>('query_timeline', {
        collectorId: collectorId || undefined,
        textQuery: textQuery || undefined,
        startTime,
        endTime,
      });

      setResults(entries);
    } catch (error) {
      console.error('Timeline query failed:', error);
      setResults([]);
    } finally {
      setIsLoading(false);
    }
  }

  function handleReset(): void {
    setStartDate('');
    setEndDate('');
    setTextQuery('');
    setResults([]);
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
            <label className="block text-sm font-medium text-[#F9FAFB]">Search</label>
            <div className="relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-[#9CA3AF] w-5 h-5" />
              <Input
                type="text"
                placeholder="Enter search query..."
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
            <div className="space-y-3">
              {results.map((entry) => (
                <div
                  key={entry.uuid}
                  className="flex items-start gap-4 p-4 rounded-lg bg-[#232B3D] hover:bg-[#2A3142] transition-colors"
                >
                  <div className="pt-1">
                    {getModalityIcon(entry.modality)}
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 mb-1">
                      <span className="inline-flex items-center px-2 py-1 rounded text-xs font-medium bg-[#4C8BF5]/20 text-[#4C8BF5]">
                        {entry.modality || 'Unknown'}
                      </span>
                      <span className="text-xs text-[#9CA3AF]">
                        {formatTimestamp(entry.timestamp)}
                      </span>
                    </div>
                    <p className="text-sm text-[#F9FAFB] truncate" title={entry.uuid}>
                      {entry.uuid.substring(0, 16)}...
                    </p>
                    <p className="text-xs text-[#9CA3AF] truncate" title={entry.origin}>
                      {entry.origin}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
