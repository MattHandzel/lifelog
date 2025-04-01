import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from './ui/card';
import { Button } from './ui/button';
import { Activity } from 'lucide-react';

interface Process {
  pid: number;
  parent_pid: number;
  name: string;
  executable: string;
  command: string;
  status: string;
  cpu_usage: number;
  memory: number;
  runtime: number;
  user: string;
  start_time: number;
}

export default function ProcessesDashboard() {
  const [processes, setProcesses] = useState<Process[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [autoRefresh, setAutoRefresh] = useState(false);
  const [sortField, setSortField] = useState<keyof Process>('cpu_usage');
  const [sortDirection, setSortDirection] = useState<'asc' | 'desc'>('desc');
  const [filterQuery, setFilterQuery] = useState('');

  useEffect(() => {
    loadProcesses();
    let intervalId: number | undefined;
    if (autoRefresh) {
      intervalId = window.setInterval(loadProcesses, 5000);
    }
    return () => {
      if (intervalId !== undefined) {
        clearInterval(intervalId);
      }
    };
  }, [autoRefresh]);

  async function loadProcesses() {
    setIsLoading(true);
    try {
      const result = await invoke<Process[]>('get_current_processes');
      setProcesses(result);
    } catch (error) {
      console.error('Failed to load processes:', error);
    } finally {
      setIsLoading(false);
    }
  }

  function toggleSort(field: keyof Process) {
    if (field === sortField) {
      setSortDirection(sortDirection === 'asc' ? 'desc' : 'asc');
    } else {
      setSortField(field);
      setSortDirection('desc');
    }
  }

  function formatMemory(bytes: number): string {
    if (bytes < 1024) return bytes + ' B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  const filteredProcesses = processes
    .filter(process => {
      if (!filterQuery) return true;
      const query = filterQuery.toLowerCase();
      return (
        process.name.toLowerCase().includes(query) ||
        process.executable?.toLowerCase().includes(query) ||
        process.command.toLowerCase().includes(query) ||
        process.pid.toString().includes(query)
      );
    })
    .sort((a, b) => {
      const aValue = a[sortField];
      const bValue = b[sortField];
      
      if (typeof aValue === 'number' && typeof bValue === 'number') {
        return sortDirection === 'asc' ? aValue - bValue : bValue - aValue;
      }
      
      const aString = String(aValue || '');
      const bString = String(bValue || '');
      
      return sortDirection === 'asc' 
        ? aString.localeCompare(bString)
        : bString.localeCompare(aString);
    });

  return (
    <div className="p-6 md:p-8 space-y-6">
      <div className="mb-8">
        <div className="flex items-center gap-3 mb-2">
          <Activity className="w-8 h-8 text-[#4C8BF5]" />
          <h1 className="title">Process Monitor</h1>
        </div>
        <p className="subtitle">View and monitor running processes</p>
      </div>

      <div className="card">
        <div className="p-6">
          <div className="flex flex-col md:flex-row gap-4 mb-8">
            <Button 
              onClick={loadProcesses} 
              className="btn-primary"
            >
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
            
            <div className="flex items-center gap-2 px-3 py-2 bg-[#1C2233] rounded-lg">
              <input
                type="checkbox"
                id="auto-refresh"
                checked={autoRefresh}
                onChange={(e) => setAutoRefresh(e.target.checked)}
                className="h-4 w-4 text-[#4C8BF5] rounded focus:ring-2 focus:ring-[#4C8BF5]/20 bg-[#232B3D] border-[#2A3142]"
              />
              <label htmlFor="auto-refresh" className="text-sm font-medium text-[#F9FAFB]">Auto-refresh (5s)</label>
            </div>
            
            <div className="flex-1 relative">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                className="h-5 w-5 absolute left-3 top-1/2 transform -translate-y-1/2 text-[#9CA3AF]"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                />
              </svg>
              <input
                type="text"
                placeholder="Filter processes by name, PID, or command..."
                className="input w-full pl-10"
                value={filterQuery}
                onChange={(e) => setFilterQuery(e.target.value)}
              />
            </div>
          </div>

          {isLoading && !autoRefresh ? (
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
              <p>Loading processes...</p>
            </div>
          ) : (
            <>
              {filteredProcesses.length === 0 ? (
                <div className="flex flex-col items-center justify-center py-12 text-[#9CA3AF]">
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    className="h-12 w-12 mb-4 text-[#4C8BF5]"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                    />
                  </svg>
                  <p className="mb-2">No processes found</p>
                  <p className="text-sm max-w-md text-center">
                    {filterQuery ? 'Try using a different filter query.' : 'Process information will appear here when available.'}
                  </p>
                </div>
              ) : (
                <div className="overflow-hidden rounded-lg border border-[#2A3142]">
                  <div className="overflow-x-auto">
                    <table className="w-full border-collapse">
                      <thead>
                        <tr className="bg-[#1C2233] border-b border-[#2A3142]">
                          <th 
                            className="p-3 font-medium text-[#F9FAFB] cursor-pointer hover:bg-[#232B3D] transition-colors" 
                            onClick={() => toggleSort('pid')}
                          >
                            <div className="flex items-center">
                              PID
                              {sortField === 'pid' && (
                                <svg
                                  xmlns="http://www.w3.org/2000/svg"
                                  className={`ml-1 h-4 w-4 transition-transform ${sortDirection === 'asc' ? 'rotate-180' : ''}`}
                                  fill="none"
                                  viewBox="0 0 24 24"
                                  stroke="currentColor"
                                >
                                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                                </svg>
                              )}
                            </div>
                          </th>
                          <th 
                            className="p-3 font-medium text-[#F9FAFB] cursor-pointer hover:bg-[#232B3D] transition-colors" 
                            onClick={() => toggleSort('name')}
                          >
                            <div className="flex items-center">
                              Name
                              {sortField === 'name' && (
                                <svg
                                  xmlns="http://www.w3.org/2000/svg"
                                  className={`ml-1 h-4 w-4 transition-transform ${sortDirection === 'asc' ? 'rotate-180' : ''}`}
                                  fill="none"
                                  viewBox="0 0 24 24"
                                  stroke="currentColor"
                                >
                                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                                </svg>
                              )}
                            </div>
                          </th>
                          <th 
                            className="p-3 font-medium text-[#F9FAFB] cursor-pointer hover:bg-[#232B3D] transition-colors" 
                            onClick={() => toggleSort('cpu_usage')}
                          >
                            <div className="flex items-center">
                              CPU %
                              {sortField === 'cpu_usage' && (
                                <svg
                                  xmlns="http://www.w3.org/2000/svg"
                                  className={`ml-1 h-4 w-4 transition-transform ${sortDirection === 'asc' ? 'rotate-180' : ''}`}
                                  fill="none"
                                  viewBox="0 0 24 24"
                                  stroke="currentColor"
                                >
                                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                                </svg>
                              )}
                            </div>
                          </th>
                          <th 
                            className="p-3 font-medium text-[#F9FAFB] cursor-pointer hover:bg-[#232B3D] transition-colors" 
                            onClick={() => toggleSort('memory')}
                          >
                            <div className="flex items-center">
                              Memory
                              {sortField === 'memory' && (
                                <svg
                                  xmlns="http://www.w3.org/2000/svg"
                                  className={`ml-1 h-4 w-4 transition-transform ${sortDirection === 'asc' ? 'rotate-180' : ''}`}
                                  fill="none"
                                  viewBox="0 0 24 24"
                                  stroke="currentColor"
                                >
                                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                                </svg>
                              )}
                            </div>
                          </th>
                          <th className="p-3 font-medium text-[#F9FAFB]">Status</th>
                        </tr>
                      </thead>
                      <tbody className="divide-y divide-[#2A3142]">
                        {filteredProcesses.map((process) => (
                          <tr key={process.pid} className="bg-[#232B3D] hover:bg-[#2A3142] transition-colors">
                            <td className="p-3 font-mono text-sm text-[#F9FAFB]">{process.pid}</td>
                            <td className="p-3 font-medium text-[#F9FAFB]">{process.name}</td>
                            <td className="p-3">
                              <div className="flex items-center">
                                <div className="w-16 bg-[#1C2233] rounded-full h-2.5 mr-2">
                                  <div 
                                    className="bg-[#4C8BF5] h-2.5 rounded-full" 
                                    style={{ width: `${Math.min(process.cpu_usage * 4, 100)}%` }}
                                  ></div>
                                </div>
                                <span className="text-[#F9FAFB]">{process.cpu_usage.toFixed(1)}%</span>
                              </div>
                            </td>
                            <td className="p-3">
                              <div className="flex items-center">
                                <div className="w-16 bg-[#1C2233] rounded-full h-2.5 mr-2">
                                  <div 
                                    className="bg-[#22C55E] h-2.5 rounded-full" 
                                    style={{ width: `${Math.min(process.memory / 1048576, 100)}%` }}
                                  ></div>
                                </div>
                                <span className="text-[#F9FAFB]">{formatMemory(process.memory)}</span>
                              </div>
                            </td>
                            <td className="p-3">
                              <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium 
                                ${process.status === 'Running' 
                                  ? 'bg-[#22C55E]/10 text-[#22C55E]' 
                                  : 'bg-[#9CA3AF]/10 text-[#9CA3AF]'}`}
                              >
                                {process.status}
                              </span>
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </div>
              )}
            </>
          )}
        </div>
      </div>
    </div>
  );
} 