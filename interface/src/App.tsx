import { useEffect, useState } from 'react';
import { BrowserRouter } from 'react-router-dom';
import { Login } from './components/Login';
import SearchDashboard from './components/SearchDashboard';
import AccountDashboard from './components/AccountDashboard';
import { LayoutDashboard, Settings, Search, User } from "lucide-react";
import FeatureTabs from "./components/FeatureTabs.tsx";
import { cn } from "./lib/utils";

import './App.css';

type View = "dashboard" | "search" | "account" | "settings";

function AppLayout() {
  const [currentView, setCurrentView] = useState<View>("dashboard");

  const NavLink = ({
    view,
    label,
    icon: Icon,
  }: {
    view: View;
    label: string;
    icon: React.ElementType;
  }) => (
    <button
      onClick={() => setCurrentView(view)}
      className={cn("nav-link w-full", currentView === view && "nav-link-active")}
    >
      <Icon className="w-5 h-5" />
      <span className="text-base font-medium">{label}</span>
    </button>
  );

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
            <NavLink view="dashboard" label="Dashboard" icon={LayoutDashboard} />
            <NavLink view="search" label="Search" icon={Search} />
            <NavLink view="account" label="Account" icon={User} />
          </div>
          {/* Settings pinned at the bottom */}
          <div className="mt-auto pt-4 border-t border-[#232B3D] flex flex-col gap-2">
            <NavLink view="settings" label="Settings" icon={Settings} />
          </div>
        </nav>
      </aside>

      {/* Main Content */}
      <main className="flex-1 flex flex-col overflow-hidden bg-[#0F111A]">
        <div className="flex-1 overflow-auto">
          {currentView === "dashboard" && <FeatureTabs />}
          {currentView === "search" && <SearchDashboard />}
          {currentView === "account" && <AccountDashboard />}
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

function App() {
  const [loading, setLoading] = useState(true);
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const checkAuth = () => {
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
    };

    checkAuth();
  }, []);

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center h-screen bg-[#0F1629] text-white p-4">
        <h1 className="text-xl font-bold mb-4">Something went wrong</h1>
        <p className="text-red-400 mb-4">{error}</p>
        <button 
          className="px-4 py-2 bg-blue-600 rounded-md hover:bg-blue-700"
          onClick={() => window.location.reload()}
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
