import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { 
  LayoutDashboard, 
  Search, 
  Camera, 
  Settings, 
  Clock,
  Shield,
  Activity,
  Menu,
  ChevronLeft
} from "lucide-react";
import { cn } from "./lib/utils";
import { client } from "./lib/client";
import { GetStateRequest } from "./gen/lifelog_pb";

// Components
import ScreenDashboard from "./components/ScreenDashboard";
import TimelineDashboard from "./components/TimelineDashboard";
import SettingsDashboard from "./components/SettingsDashboard";
import CameraDashboard from "./components/CameraDashboard";
import SearchDashboard from "./components/SearchDashboard";

type View = "overview" | "screen" | "timeline" | "search" | "camera" | "settings";

function App() {
  const [currentView, setCurrentView] = useState<View>("screen");
  const [isSidebarOpen, setIsSidebarOpen] = useState(true);

  const { data: systemState, error: stateError } = useQuery({
    queryKey: ['systemState'],
    queryFn: () => client.getState(new GetStateRequest()),
    refetchInterval: 5000,
  });

  const isOnline = !stateError && !!systemState;

  return (
    <div className="flex h-screen bg-slate-950 text-slate-100 overflow-hidden font-sans">
      {/* Sidebar */}
      <aside className={cn(
        "bg-slate-900 border-r border-slate-800 transition-all duration-300 flex flex-col",
        isSidebarOpen ? "w-64" : "w-20"
      )}>
        <div className="p-6 flex items-center gap-3 border-b border-slate-800">
          <div className="w-8 h-8 bg-blue-600 rounded-lg flex items-center justify-center shrink-0">
            <Activity className="w-5 h-5 text-white" />
          </div>
          {isSidebarOpen && <h1 className="font-bold text-xl tracking-tight">LifeLog</h1>}
        </div>

        <nav className="flex-1 p-4 space-y-2">
          <NavItem 
            icon={LayoutDashboard} 
            label="Overview" 
            active={currentView === 'overview'} 
            onClick={() => setCurrentView('overview')}
            collapsed={!isSidebarOpen}
          />
          <NavItem 
            icon={Camera} 
            label="Screenshots" 
            active={currentView === 'screen'} 
            onClick={() => setCurrentView('screen')}
            collapsed={!isSidebarOpen}
          />
          <NavItem 
            icon={Camera} 
            label="Camera" 
            active={currentView === 'camera'} 
            onClick={() => setCurrentView('camera')}
            collapsed={!isSidebarOpen}
          />
          <NavItem 
            icon={Clock} 
            label="Timeline" 
            active={currentView === 'timeline'} 
            onClick={() => setCurrentView('timeline')}
            collapsed={!isSidebarOpen}
          />
          <NavItem 
            icon={Search} 
            label="Search" 
            active={currentView === 'search'} 
            onClick={() => setCurrentView('search')}
            collapsed={!isSidebarOpen}
          />
        </nav>

        <div className="p-4 border-t border-slate-800 space-y-2">
          <div className={cn(
            "flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-colors",
            isOnline ? "text-green-500 bg-green-500/5" : "text-red-500 bg-red-500/5"
          )}>
            <Shield className="w-4 h-4 shrink-0" />
            {isSidebarOpen && <span>{isOnline ? 'System Online' : 'System Offline'}</span>}
          </div>
          <NavItem 
            icon={Settings} 
            label="Settings" 
            active={currentView === 'settings'} 
            onClick={() => setCurrentView('settings')}
            collapsed={!isSidebarOpen}
          />
          <button 
            onClick={() => setIsSidebarOpen(!isSidebarOpen)}
            className="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition-all"
          >
            {isSidebarOpen ? <ChevronLeft className="w-4 h-4" /> : <Menu className="w-4 h-4" />}
            {isSidebarOpen && <span>Collapse</span>}
          </button>
        </div>
      </aside>

      {/* Main Area */}
      <main className="flex-1 flex flex-col min-w-0 bg-slate-950">
        <div className="flex-1 overflow-auto">
          {currentView === 'screen' && <ScreenDashboard collectorId={null} />}
          {currentView === 'camera' && <CameraDashboard />}
          {currentView === 'timeline' && <TimelineDashboard collectorId={null} />}
          {currentView === 'search' && <SearchDashboard />}
          {currentView === 'overview' && <OverviewView />}
          {currentView === 'settings' && <SettingsDashboard />}
        </div>
      </main>
    </div>
  )
}

function NavItem({ icon: Icon, label, active, onClick, collapsed }: any) {
  return (
    <button
      onClick={onClick}
      className={cn(
        "w-full flex items-center gap-3 px-3 py-2.5 rounded-xl transition-all duration-200 group",
        active 
          ? "bg-blue-600 text-white shadow-lg shadow-blue-600/20" 
          : "text-slate-400 hover:text-white hover:bg-slate-800"
      )}
    >
      <Icon className={cn("w-5 h-5 shrink-0", active ? "text-white" : "group-hover:text-blue-400")} />
      {!collapsed && <span className="font-medium text-sm">{label}</span>}
    </button>
  )
}

function OverviewView() {
  return (
    <div className="p-8 space-y-8 max-w-6xl mx-auto">
      <header>
        <h1 className="text-3xl font-bold mb-2">Welcome Back</h1>
        <p className="text-slate-400">Here's what's happening across your devices.</p>
      </header>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div className="bg-slate-900 p-6 rounded-2xl border border-slate-800 shadow-lg">
          <p className="text-slate-500 text-sm font-medium mb-1">Active Streams</p>
          <span className="text-3xl font-bold text-blue-500">Live</span>
        </div>
      </div>
    </div>
  )
}

export default App
