import { useState, useEffect } from 'react';
import { Search, Filter, ArrowUpDown, Image, FileAudio, File, Loader, Calendar } from 'lucide-react';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { Card, CardContent } from './ui/card';

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

interface SearchResult {
  id: string;
  type: 'image' | 'audio' | 'file';
  name: string;
  path: string;
  timestamp: number;
  source: string; 
  preview?: string;
  size?: number;
  duration?: number; 
  metadata?: Record<string, any>; 
}

export default function SearchDashboard() {
  const [searchQuery, setSearchQuery] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [fileTypeFilter, setFileTypeFilter] = useState<string>('all');
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('desc');
  const [page] = useState(1);
  const [totalPages] = useState(1);
  const [sourceFilter, setSourceFilter] = useState<string>('all');

  const performSearch = async (_resetPage = true) => {
    console.warn('Search: not yet implemented via gRPC');
    setResults([]);
    setIsLoading(false);
  };

  const loadMoreResults = async () => {
    console.warn('Search load more: not yet implemented via gRPC');
    setIsLoading(false);
  };

  useEffect(() => {
    performSearch();
  }, [fileTypeFilter, sortOrder, sourceFilter]);

  const handleSearchSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    performSearch();
  };

  const formatDate = (timestamp: number): string => {
    return new Date(timestamp).toLocaleString();
  };

  const formatFileSize = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  };

  const renderTypeIcon = (type: string) => {
    switch (type) {
      case 'image':
        return <Image className="w-5 h-5 text-blue-400" />;
      case 'audio':
        return <FileAudio className="w-5 h-5 text-green-400" />;
      default:
        return <File className="w-5 h-5 text-gray-400" />;
    }
  };

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
                  <SelectItem value="camera">Camera</SelectItem>
                  <SelectItem value="screen">Screenshots</SelectItem>
                  <SelectItem value="microphone">Microphone</SelectItem>
                  <SelectItem value="text">Files</SelectItem>
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
                <Card key={result.id} className="bg-[#1A1E2E] border-[#232B3D] overflow-hidden">
                  <div className="relative">
                    {result.type === 'image' && result.preview && (
                      <div className="aspect-video bg-[#0F111A] flex items-center justify-center overflow-hidden">
                        <img 
                          src={result.preview} 
                          alt={result.name} 
                          className="w-full h-full object-cover"
                        />
                      </div>
                    )}
                    {result.type === 'audio' && (
                      <div className="aspect-video bg-[#0F111A] flex items-center justify-center">
                        <FileAudio className="w-16 h-16 text-[#4C8BF5]" />
                      </div>
                    )}
                    {result.type === 'file' && (
                      <div className="aspect-video bg-[#0F111A] flex items-center justify-center">
                        <File className="w-16 h-16 text-[#4C8BF5]" />
                      </div>
                    )}
                  </div>
                  <CardContent className="p-4">
                    <div className="flex items-start gap-3">
                      <div className="mt-1">
                        {renderTypeIcon(result.type)}
                      </div>
                      <div className="flex-1 min-w-0">
                        <h3 className="font-medium text-[#F9FAFB] truncate" title={result.name}>
                          {result.name}
                        </h3>
                        <div className="text-sm text-[#9CA3AF] mt-1">
                          <p className="truncate" title={result.path}>
                            Source: {result.source}
                          </p>
                          <p>{formatDate(result.timestamp)}</p>
                          {result.size && (
                            <p>{formatFileSize(result.size)}</p>
                          )}
                          {result.duration && (
                            <p>{Math.floor(result.duration / 60)}:{(result.duration % 60).toString().padStart(2, '0')}</p>
                          )}
                        </div>
                      </div>
                    </div>
                  </CardContent>
                </Card>
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