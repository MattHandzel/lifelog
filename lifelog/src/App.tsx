import { useState } from "react";
import { cn } from "./lib/utils";
import { LayoutDashboard, Activity, Camera, Settings } from "lucide-react";
import FeatureTabs from "./components/FeatureTabs";

type View = "dashboard" | "settings";

function App() {
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

        {/* Navigation */}
        <nav className="flex-1 flex flex-col">
          <div className="flex flex-col gap-2 mt-4">
            <NavLink view="dashboard" label="Dashboard" icon={LayoutDashboard} />
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
