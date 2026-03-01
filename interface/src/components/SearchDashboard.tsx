import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Search, Filter, ArrowUpDown, Loader, Calendar } from 'lucide-react';
import { Button } from './ui/button';
import { Input } from './ui/input';
import ResultCard, { type FrameDataWrapper, type SearchResult } from './ResultCard';

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from './ui/dropdown-menu';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from './ui/select';

const MODALITY_TYPE_MAP: Record<string, SearchResult['type']> = {
  screen: 'image',
  camera: 'image',
  audio: 'audio',
  microphone: 'audio',
  ocr: 'file',
  browser: 'file',
  keystrokes: 'file',
  clipboard: 'file',
  shell_history: 'file',
  shellhistory: 'file',
  window_activity: 'file',
  windowactivity: 'file',
  mouse: 'file',
  processes: 'file',
  weather: 'file',
  hyprland: 'file',
};

interface TimelineEntry {
  uuid: string;
  origin: string;
  modality: string;
  timestamp: number | null;
}

interface LifelogDataKeyWrapper {
  uuid: string;
  origin: string;
}

function extractSnippetSource(frame: FrameDataWrapper | undefined): string {
  if (!frame) return '';
  return [
    frame.text ?? '',
    frame.title ?? '',
    frame.url ?? '',
    frame.command ?? '',
    frame.window_title ?? '',
    frame.application ?? '',
    frame.working_dir ?? '',
  ].join(' ').replace(/\s+/g, ' ').trim();
}

function createSnippet(raw: string, queryTerms: string[]): string | undefined {
  const text = raw.trim();
  if (!text) return undefined;
  const maxChars = 180;
  if (queryTerms.length === 0) {
    return text.length > maxChars ? `${text.slice(0, maxChars)}...` : text;
  }
  const lower = text.toLowerCase();
  const firstHit = queryTerms
    .map((term) => ({ term, index: lower.indexOf(term.toLowerCase()) }))
    .filter((hit) => hit.index >= 0)
    .sort((a, b) => a.index - b.index)[0];
  if (!firstHit) {
    return text.length > maxChars ? `${text.slice(0, maxChars)}...` : text;
  }
  const start = Math.max(0, firstHit.index - 55);
  const end = Math.min(text.length, firstHit.index + firstHit.term.length + 95);
  const prefix = start > 0 ? '...' : '';
  const suffix = end < text.length ? '...' : '';
  return `${prefix}${text.slice(start, end)}${suffix}`;
}

function buildSearchResult(
  entry: TimelineEntry,
  frame: FrameDataWrapper | undefined,
  queryTerms: string[],
): SearchResult {
  const modality = (frame?.modality ?? entry.modality).trim();
  const modalityLower = modality.toLowerCase();
  const timestampSeconds = frame?.timestamp ?? entry.timestamp ?? 0;
  const snippet = createSnippet(extractSnippetSource(frame), queryTerms);
  return {
    id: entry.uuid,
    type: MODALITY_TYPE_MAP[modalityLower] ?? 'file',
    name: frame?.title ?? frame?.window_title ?? entry.uuid.substring(0, 8),
    path: frame?.url ?? frame?.working_dir ?? entry.origin,
    timestamp: timestampSeconds * 1000,
    source: entry.origin,
    modality,
    preview: frame?.image_data_url ?? undefined,
    duration: frame?.audio_duration_secs ?? frame?.duration_secs ?? undefined,
    metadata: {
      url: frame?.url ?? null,
      title: frame?.title ?? null,
      command: frame?.command ?? null,
      application: frame?.application ?? null,
      visit_count: frame?.visit_count ?? null,
      processes_count: frame?.processes?.length ?? 0,
    },
    snippet,
    highlightTerms: queryTerms,
  };
}

function matchesSourceFilter(result: SearchResult, sourceFilter: string): boolean {
  if (sourceFilter === 'all') return true;
  const sourceLower = result.source.toLowerCase();
  const modalityLower = result.modality.toLowerCase();
  return sourceLower.includes(sourceFilter) || modalityLower.includes(sourceFilter);
}

export default function SearchDashboard(): JSX.Element {
  const [searchQuery, setSearchQuery] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [fileTypeFilter, setFileTypeFilter] = useState<string>('all');
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('desc');
  const [page] = useState(1);
  const [totalPages] = useState(1);
  const [sourceFilter, setSourceFilter] = useState<string>('all');

  async function performSearch(_resetPage = true): Promise<void> {
    if (!searchQuery.trim()) return;
    setIsLoading(true);
    try {
      const queryTerms = searchQuery
        .trim()
        .split(/\s+/)
        .map((term) => term.toLowerCase())
        .filter(Boolean);
      const entries = await invoke<TimelineEntry[]>('query_timeline', {
        textQuery: [searchQuery],
      });
      const keys: LifelogDataKeyWrapper[] = entries.map((entry) => ({
        uuid: entry.uuid,
        origin: entry.origin,
      }));
      const frames = keys.length > 0
        ? await invoke<FrameDataWrapper[]>('get_frame_data_thumbnails', { keys })
        : [];
      const frameByUuid = new Map(frames.map((frame) => [frame.uuid, frame]));
      const mapped = entries.map((entry) => buildSearchResult(entry, frameByUuid.get(entry.uuid), queryTerms));
      const filtered = mapped
        .filter((result) => fileTypeFilter === 'all' || result.type === fileTypeFilter)
        .filter((result) => matchesSourceFilter(result, sourceFilter))
        .sort((a, b) => sortOrder === 'desc' ? b.timestamp - a.timestamp : a.timestamp - b.timestamp);
      setResults(filtered);
    } catch (error) {
      console.error('Search failed:', error);
      setResults([]);
    } finally {
      setIsLoading(false);
    }
  }

  async function loadMoreResults(): Promise<void> {
    // Pagination comes later
    console.log('Load more: pagination not yet implemented');
  }

  useEffect(function () {
    performSearch();
  }, [fileTypeFilter, sortOrder, sourceFilter]);

  function handleSearchSubmit(e: React.FormEvent): void {
    e.preventDefault();
    performSearch();
  }

  return (
    <div className="p-6 md:p-8 space-y-6">
      <div className="mb-8">
        <div className="flex items-center gap-3 mb-2">
          <Search className="w-8 h-8 text-[#4C8BF5]" />
          <h1 className="title">Search</h1>
        </div>
        <p className="subtitle">Search across all your captured data</p>
      </div>

      {/* Search Input and Filters */}
      <div className="card mb-6">
        <div className="p-6 space-y-4">
          <form onSubmit={handleSearchSubmit} className="flex gap-2">
            <div className="relative flex-1">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-[#9CA3AF] w-5 h-5" />
              <Input
                type="text"
                placeholder="Search for files, images, audio..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="pl-10 bg-[#1C2233] border-[#232B3D] text-[#F9FAFB]"
              />
            </div>
            <Button type="submit" className="bg-[#4C8BF5] hover:bg-[#3A78E7]">
              Search
            </Button>
          </form>

          <div className="flex flex-wrap gap-2 items-center justify-between border-t border-[#232B3D] pt-4">
            <div className="flex flex-wrap gap-2 items-center">
              <div className="flex items-center gap-2">
                <Filter className="w-4 h-4 text-[#9CA3AF]" />
                <span className="text-sm text-[#9CA3AF]">Filter by:</span>
              </div>

              {/* File Type Filter */}
              <Select value={fileTypeFilter} onValueChange={setFileTypeFilter}>
                <SelectTrigger className="w-[140px] bg-[#1C2233] border-[#232B3D] text-[#F9FAFB]">
                  <SelectValue placeholder="File Type" />
                </SelectTrigger>
                <SelectContent className="bg-[#1C2233] border-[#232B3D] text-[#F9FAFB]">
                  <SelectItem value="all">All Types</SelectItem>
                  <SelectItem value="image">Images</SelectItem>
                  <SelectItem value="audio">Audio</SelectItem>
                  <SelectItem value="file">Files</SelectItem>
                </SelectContent>
              </Select>

              {/* Source Filter */}
              <Select value={sourceFilter} onValueChange={setSourceFilter}>
                <SelectTrigger className="w-[140px] bg-[#1C2233] border-[#232B3D] text-[#F9FAFB]">
                  <SelectValue placeholder="Source" />
                </SelectTrigger>
                <SelectContent className="bg-[#1C2233] border-[#232B3D] text-[#F9FAFB]">
                  <SelectItem value="all">All Sources</SelectItem>
                  <SelectItem value="screen">Screen</SelectItem>
                  <SelectItem value="camera">Camera</SelectItem>
                  <SelectItem value="audio">Audio (system)</SelectItem>
                  <SelectItem value="microphone">Microphone</SelectItem>
                  <SelectItem value="browser">Browser</SelectItem>
                  <SelectItem value="keystrokes">Keystrokes</SelectItem>
                  <SelectItem value="clipboard">Clipboard</SelectItem>
                  <SelectItem value="shell_history">Shell History</SelectItem>
                  <SelectItem value="mouse">Mouse</SelectItem>
                  <SelectItem value="processes">Processes</SelectItem>
                  <SelectItem value="weather">Weather</SelectItem>
                  <SelectItem value="hyprland">Hyprland</SelectItem>
                  <SelectItem value="window_activity">Window Activity</SelectItem>
                </SelectContent>
              </Select>
            </div>

            {/* Sort Order */}
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="outline" className="bg-[#1C2233] border-[#232B3D] text-[#F9FAFB]">
                  <Calendar className="mr-2 h-4 w-4" />
                  {sortOrder === 'desc' ? 'Newest First' : 'Oldest First'}
                  <ArrowUpDown className="ml-2 h-4 w-4" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent className="bg-[#1C2233] border-[#232B3D] text-[#F9FAFB]">
                <DropdownMenuItem onClick={() => setSortOrder('desc')}>
                  Newest First
                </DropdownMenuItem>
                <DropdownMenuItem onClick={() => setSortOrder('asc')}>
                  Oldest First
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        </div>
      </div>

      {/* Results */}
      <div className="space-y-6">
        {isLoading && results.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-12">
            <Loader className="w-10 h-10 text-[#4C8BF5] animate-spin mb-4" />
            <p className="text-[#9CA3AF]">Searching...</p>
          </div>
        ) : results.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-12 text-center">
            <Search className="w-10 h-10 text-[#9CA3AF] mb-4" />
            <h3 className="text-lg font-medium text-[#F9FAFB] mb-2">No results found</h3>
            <p className="text-[#9CA3AF] max-w-md">
              {searchQuery 
                ? `No matches found for "${searchQuery}". Try different keywords or filters.` 
                : "Start by typing a search query above."}
            </p>
          </div>
        ) : (
          <>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
              {results.map((result) => (
                <ResultCard key={result.id} result={result} />
              ))}
            </div>

            {/* Pagination / Load More */}
            {page < totalPages && (
              <div className="flex justify-center mt-8">
                <Button
                  onClick={loadMoreResults}
                  disabled={isLoading}
                  className="bg-[#1C2233] border-[#232B3D] text-[#F9FAFB] hover:bg-[#232B3D]"
                >
                  {isLoading ? (
                    <>
                      <Loader className="w-4 h-4 mr-2 animate-spin" />
                      Loading...
                    </>
                  ) : (
                    'Load More'
                  )}
                </Button>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
} 
