import { useState } from "react";
import TextUploadDashboard from "./components/TextUploadDashboard";
import ProcessesDashboard from "./components/ProcessesDashboard";
import ScreenDashboard from "./components/ScreenDashboard";
import { cn } from "./lib/utils";
import { LayoutDashboard, Activity, Camera, Settings } from "lucide-react";

type View = "dashboard" | "processes" | "screenshots" | "settings";

function App() {
  const [currentView, setCurrentView] = useState<View>("dashboard");

  const NavLink = ({ view, label, icon: Icon }: { view: View; label: string; icon: React.ElementType }) => (
    <button
      onClick={() => setCurrentView(view)}
      className={cn(
        "nav-link",
        currentView === view && "nav-link-active"
      )}
    >
      <Icon className="w-5 h-5" />
      <span className="text-base font-medium">{label}</span>
    </button>
  );

  return (
    <div className="flex h-screen bg-[#0F111A] text-[#F9FAFB]">
      {/* Sidebar */}
      <aside className="w-16 md:w-64 flex flex-col p-4 bg-[#1A1E2E] border-r border-[#232B3D]">
        <h1 className="text-2xl font-bold px-4 py-4 gradient-text hidden md:block">
          LifeLog
        </h1>
        {/* Mobile Logo */}
        <h1 className="text-2xl font-bold p-2 text-center md:hidden gradient-text">L</h1>
        <nav className="flex flex-col gap-2 mt-4">
          <NavLink view="dashboard" label="Dashboard" icon={LayoutDashboard} />
          <NavLink view="processes" label="Processes" icon={Activity} />
          <NavLink view="screenshots" label="Screenshots" icon={Camera} />
          <NavLink view="settings" label="Settings" icon={Settings} />
        </nav>
      </aside>

      {/* Main Content */}
      <main className="flex-1 flex flex-col overflow-hidden bg-[#0F111A]">
        <div className="flex-1 overflow-auto">
          {currentView === "dashboard" && <TextUploadDashboard />}
          {currentView === "processes" && <ProcessesDashboard />}
          {currentView === "screenshots" && <ScreenDashboard />}
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

export default App;
