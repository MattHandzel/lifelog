import { useEffect, useState } from 'react';
import { BrowserRouter } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';
import { Login } from './components/Login';
import SearchDashboard from './components/SearchDashboard';
import DevicesDashboard from './components/DevicesDashboard';
import { LayoutDashboard, Settings, Search, Laptop, Shield, Power } from "lucide-react";
import FeatureTabs from "./components/FeatureTabs.tsx";
import { cn } from "./lib/utils";
import { Switch } from "./components/ui/switch";

import './App.css';

type View = "dashboard" | "search" | "devices" | "settings";

interface NavLinkProps {
  view: View;
  label: string;
  icon: React.ElementType;
  currentView: View;
  onClick: (view: View) => void;
}

function NavLink({
  view,
  label,
  icon: Icon,
  currentView,
  onClick,
}: NavLinkProps): JSX.Element {
  return (
    <button
      onClick={function () { onClick(view); }}
      className={cn("nav-link w-full", currentView === view && "nav-link-active")}
    >
      <Icon className="w-5 h-5" />
      <span className="text-base font-medium">{label}</span>
    </button>
  );
}

function AppLayout(): JSX.Element {
  const [currentView, setCurrentView] = useState<View>("dashboard");
  const [isPaused, setIsPaused] = useState(false);
  const [isPausing, setIsPausing] = useState(false);

  function handleViewChange(view: View): void {
    setCurrentView(view);
  }

  async function handleGlobalPause(paused: boolean) {
    if (isPausing) return;
    setIsPausing(true);
    try {
      // 1. Get all collectors
      const ids = await invoke<string[]>('get_collector_ids');
      
      // 2. Iterate and pause/unpause everything
      // This is a "best effort" broadcast from the frontend
      const components = ['screen', 'camera', 'microphone', 'processes', 'hyprland'];
      
      for (const id of ids) {
        for (const type of components) {
          try {
            // Fetch current config to preserve other settings
            const current = await invoke<any>('get_component_config', { 
              collectorId: id, 
              componentType: type 
            });
            
            if (current) {
              await invoke('set_component_config', {
                collectorId: id,
                componentType: type,
                configValue: { ...current, enabled: !paused }
              });
            }
          } catch (e) {
            console.warn(`Failed to set ${type} for ${id}:`, e);
          }
        }
      }
      setIsPaused(paused);
    } catch (err) {
      console.error('Global pause failed:', err);
      alert('Failed to update some devices. Check device status.');
    } finally {
      setIsPausing(false);
    }
  }

  return (
    <div className="flex h-screen bg-[#0F111A] text-[#F9FAFB]">
      {/* Sidebar */}
      <aside className="w-16 md:w-64 flex flex-col p-4 bg-[#1A1E2E] border-r border-[#232B3D]">
        {/* Desktop Logo */}
        <h1 className="text-2xl font-bold px-4 py-4 gradient-text hidden md:block">
          LifeLog
        </h1>
        {/* Mobile Logo */}
        <h1 className="text-2xl font-bold p-2 text-center md:hidden gradient-text">
          L
        </h1>

        <nav className="flex-1 flex flex-col">
          <div className="flex flex-col gap-2 mt-4">
            <NavLink 
              view="dashboard" 
              label="Dashboard" 
              icon={LayoutDashboard} 
              currentView={currentView}
              onClick={handleViewChange}
            />
            <NavLink 
              view="search" 
              label="Search" 
              icon={Search} 
              currentView={currentView}
              onClick={handleViewChange}
            />
            <NavLink 
              view="devices" 
              label="Devices" 
              icon={Laptop} 
              currentView={currentView}
              onClick={handleViewChange}
            />
          </div>
          
          {/* Global Pause Control */}
          <div className="mt-auto mb-4 px-2 hidden md:block">
            <div className={cn(
              "rounded-lg p-3 border transition-colors",
              isPaused 
                ? "bg-red-500/10 border-red-500/30" 
                : "bg-[#232B3D] border-[#2A3142]"
            )}>
              <div className="flex items-center justify-between mb-2">
                <div className="flex items-center gap-2">
                  <Shield className={cn("w-4 h-4", isPaused ? "text-red-500" : "text-[#4C8BF5]")} />
                  <span className="text-sm font-medium">Recording</span>
                </div>
                <Switch 
                  checked={!isPaused}
                  onCheckedChange={(checked) => handleGlobalPause(!checked)}
                  disabled={isPausing}
                  className={cn(
                    "data-[state=checked]:bg-[#4C8BF5] data-[state=unchecked]:bg-red-500"
                  )}
                />
              </div>
              <p className="text-xs text-[#9CA3AF]">
                {isPausing ? "Updating..." : isPaused ? "All capture paused" : "System active"}
              </p>
            </div>
          </div>

          {/* Settings pinned at the bottom */}
          <div className="pt-4 border-t border-[#232B3D] flex flex-col gap-2">
            <NavLink 
              view="settings" 
              label="Settings" 
              icon={Settings} 
              currentView={currentView}
              onClick={handleViewChange}
            />
          </div>
        </nav>
      </aside>

      {/* Main Content */}
      <main className="flex-1 flex flex-col overflow-hidden bg-[#0F111A]">
        <div className="flex-1 overflow-auto">
          {currentView === "dashboard" && <FeatureTabs />}
          {currentView === "search" && <SearchDashboard />}
          {currentView === "devices" && <DevicesDashboard />}
          {currentView === "settings" && (
            <div className="p-8">
              <h2 className="title mb-4">Settings</h2>
              <p className="subtitle mb-8">Manage your application preferences</p>
              <div className="space-y-6">
                <div className="card p-4">
                  <div>
                    <h3 className="font-medium text-[#F9FAFB]">Version</h3>
                    <p className="text-sm text-[#9CA3AF]">Lifelog Interface • Version 0.1.0</p>
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>
        
        {/* Footer */}
        <footer className="border-t border-[#232B3D] py-4 px-8 text-center text-sm text-[#9CA3AF] bg-[#1A1E2E]">
          <p>Lifelog Interface • Version 0.1.0</p>
        </footer>
      </main>
    </div>
  );
}

function App(): JSX.Element {
  const [loading, setLoading] = useState(true);
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(function () {
    function checkAuth(): void {
      try {
        // For development: bypass authentication
        const authenticated = true;
        setIsAuthenticated(authenticated);
        setLoading(false);
      } catch (err) {
        console.error('[APP] Auth check error:', err);
        setError('Authentication check failed');
        setLoading(false);
      }
    }

    checkAuth();
  }, []);

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center h-screen bg-[#0F1629] text-white p-4">
        <h1 className="text-xl font-bold mb-4">Something went wrong</h1>
        <p className="text-red-400 mb-4">{error}</p>
        <button 
          className="px-4 py-2 bg-blue-600 rounded-md hover:bg-blue-700"
          onClick={function () { window.location.reload(); }}
        >
          Reload Application
        </button>
      </div>
    );
  }

  if (loading) {
    return (
      <div className="flex flex-col items-center justify-center h-screen bg-[#0F1629] text-white">
        <div className="text-center space-y-6 max-w-md px-6">
          {/* Logo */}
          <div className="mb-8">
            <h1 className="text-5xl font-bold gradient-text tracking-tight mb-2">LifeLog</h1>
          </div>
          
          {/* Loading text */}
          <h2 className="text-2xl font-medium text-[#F9FAFB]">
            Loading Lifelog Dashboard...
          </h2>
          
          {/* Loading indicator */}
          <div className="w-full h-1.5 bg-[#232B3D] rounded-full overflow-hidden mt-8 mb-4">
            <div className="h-full bg-[#4C8BF5] rounded-full animate-loading-progress"></div>
          </div>
          
          {/* Error message */}
          <p className="text-[#9CA3AF] text-sm mt-4">
            If this message persists, there may be an issue with the application.
          </p>
        </div>
        
        {/* Version footer */}
        <div className="absolute bottom-8 text-center">
          <p className="text-sm text-[#9CA3AF]">Lifelog Interface • Version 0.1.0</p>
        </div>
      </div>
    );
  }
  
  if (!isAuthenticated) {
    return (
      <BrowserRouter>
        <div className="flex items-center justify-center h-screen bg-[#0F1629] text-white">
          <Login />
        </div>
      </BrowserRouter>
    );
  }

  return (
    <BrowserRouter>
      <AppLayout />
    </BrowserRouter>
  );
}

export default App;

