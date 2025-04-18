import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { open as openFile } from '@tauri-apps/plugin-shell';
import { Button } from './ui/button';
import { FileIcon, PlusIcon, Loader2Icon, FileTextIcon, FolderIcon, Search } from 'lucide-react';
import { cn } from '../lib/utils';
import { useDebounce } from '../lib/hooks';

interface TextFile {
  filename: string;
  original_path: string;
  file_type: string;
  file_size: number;
  stored_path: string;
  content_hash: string;
}

export default function TextUploadDashboard() {
  const [files, setFiles] = useState<TextFile[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [isUploading, setIsUploading] = useState(false);
  const [tauriReady, setTauriReady] = useState(false);
  
  const debouncedSearch = useDebounce(searchQuery, 300);

  useEffect(() => {
    const checkTauriAvailable = async () => {
      try {
        await invoke('get_all_text_files').catch(() => {
          console.log('Initial invoke test failed, but that\'s okay');
        });
        setTauriReady(true);
        console.log('Tauri API is ready');
        loadFiles();
      } catch (error) {
        console.log('Tauri API not ready yet, trying again in 500ms');
        setTimeout(checkTauriAvailable, 500);
      }
    };
    
    checkTauriAvailable();
  }, []);

  useEffect(() => {
    if (debouncedSearch !== undefined && tauriReady) {
      handleSearch();
    }
  }, [debouncedSearch, tauriReady]);

  async function loadFiles() {
    if (!tauriReady) return;
    
    setIsLoading(true);
    try {
      const result = await invoke<TextFile[]>('get_all_text_files');
      setFiles(result || []);
    } catch (error) {
      console.error('Failed to load files:', error);
    } finally {
      setIsLoading(false);
    }
  }

  async function handleFileUpload() {
    if (!tauriReady) {
      console.error('Tauri API not initialized yet');
      alert('Application is still initializing. Please try again in a moment.');
      return;
    }
    
    try {
      console.log('Starting file upload process...');
      
      const selected = await open({
        directory: false,
        multiple: false,
        filters: [{
          name: 'Text Files',
          extensions: ['txt', 'md', 'json', 'csv']
        }]
      });

      console.log('File selection result:', selected);

      if (!selected) {
        console.log('User cancelled the file selection');
        return;
      }

      setIsUploading(true);
      try {
        const result = await invoke<TextFile>('upload_text_file', { filePath: selected });
        console.log('File uploaded successfully:', result);
        await loadFiles();
      } catch (error) {
        console.error('Upload failed - detailed error:', error);
        alert(`Failed to upload file: ${error}`);
      }
    } catch (error) {
      console.error('File selection failed - detailed error:', error);
      alert('Failed to select file. Please try again.');
    } finally {
      setIsUploading(false);
    }
  }

  async function handleSearch() {
    if (!tauriReady) return;
    
    if (!searchQuery.trim()) {
      return loadFiles();
    }
    
    setIsLoading(true);
    try {
      const result = await invoke<TextFile[]>('search_text_files', { pattern: searchQuery });
      setFiles(result || []);
    } catch (error) {
      console.error('Search failed:', error);
    } finally {
      setIsLoading(false);
    }
  }

  async function handleOpenFile(path: string) {
    if (!tauriReady) return;
    
    try {
      await openFile(path);
    } catch (error) {
      console.error('Failed to open file:', error);
      alert('Failed to open file. Please try again.');
    }
  }

  function formatFileSize(bytes: number): string {
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return (bytes / Math.pow(1024, i)).toFixed(1) + ' ' + sizes[i];
  }

  return (
    <div className="p-6 md:p-8">
      <div className="mb-8">
        <div className="flex items-center gap-3 mb-2">
          <FolderIcon className="w-8 h-8 text-[#4C8BF5]" />
          <h1 className="title">Text Uploads</h1>
        </div>
        <p className="subtitle">Upload, search, and open files</p>
      </div>

      <div className="flex flex-col sm:flex-row justify-between gap-4 mb-8">
        <div className="relative flex-1">
          <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
            <Search className="h-5 w-5 text-[#9CA3AF]" />
          </div>
          <input
            type="text"
            placeholder="Search files..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="input w-full pl-10 placeholder:text-[#9CA3AF]"
            style={{ textIndent: "1.5rem" }}
          />
        </div>
        <div className="flex items-center gap-2">
          <Button
            onClick={handleFileUpload}
            className="btn-primary"
            disabled={isUploading || !tauriReady}
          >
            {isUploading ? (
              <Loader2Icon className="h-5 w-5 mr-2 animate-spin" />
            ) : (
              <PlusIcon className="h-5 w-5 mr-2" />
            )}
            {isUploading ? 'Uploading...' : 'Upload File'}
          </Button>
        </div>
      </div>

      <div className="card">
        <div className="p-6">
          {!tauriReady ? (
            <div className="flex flex-col items-center justify-center py-12 text-[#9CA3AF]">
              <Loader2Icon className="w-8 h-8 animate-spin mb-4" />
              <p>Initializing application...</p>
            </div>
          ) : isLoading ? (
            <div className="flex flex-col items-center justify-center py-12 text-[#9CA3AF]">
              <Loader2Icon className="w-8 h-8 animate-spin mb-4" />
              <p>Loading files...</p>
            </div>
          ) : files.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-12 text-[#9CA3AF]">
              <FileTextIcon className="w-12 h-12 mb-4 animate-float" />
              <p className="mb-2">No files found</p>
              <p className="text-sm text-center max-w-md">
                {searchQuery.trim() 
                  ? 'Try adjusting your search terms'
                  : 'Upload your first file to get started'}
              </p>
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {files.map((file) => (
                <div
                  key={file.content_hash}
                  onClick={() => handleOpenFile(file.stored_path)}
                  className={cn(
                    "card card-hover flex items-center gap-4 p-4",
                    "group cursor-pointer"
                  )}
                >
                  <div className="p-3 bg-[#1C2233] rounded-lg group-hover:bg-[#232B3D] transition-colors">
                    <FileIcon className="w-6 h-6 text-[#4C8BF5]" />
                  </div>
                  <div className="flex-1 min-w-0">
                    <h3 className="text-lg font-medium text-[#F9FAFB] truncate group-hover:text-[#4C8BF5] transition-colors">
                      {file.filename}
                    </h3>
                    <div className="flex items-center gap-3 text-sm text-[#9CA3AF]">
                      <span className="uppercase px-2 py-0.5 bg-[#1C2233] rounded-md text-xs font-medium">
                        {file.file_type}
                      </span>
                      <span>{formatFileSize(file.file_size)}</span>
                    </div>
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