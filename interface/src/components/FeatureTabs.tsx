import { useState } from "react";
import { cn } from "../lib/utils";
import {
  FileTextIcon,
  CameraIcon,
  ActivityIcon,
  MicIcon,
  EyeIcon,
  MousePointerIcon,
  KeyboardIcon,
  CloudIcon,
  ThermometerIcon,
  WifiIcon,
  LayoutGridIcon,
  MonitorIcon,
  KeyboardIcon as InputIcon,
  PanelTopIcon,
  MonitorIcon as DesktopIcon,
  ChevronRightIcon,
  User as UserIcon,
  Cpu as CpuIcon,
  MemoryStick as MemoryStickIcon,
  Network as NetworkIcon,
  Disc3 as DiskIcon,
  Power as PowerIcon,
  ScreenShare,
  Clock
} from "lucide-react";
import TextUploadDashboard from "./TextUploadDashboard";
import ProcessesDashboard from "./ProcessesDashboard";
import ScreenDashboard from "./ScreenDashboard";
import CameraDashboard from "./CameraDashboard";
import MicrophoneDashboard from "./MicrophoneDashboard";
import TimelineDashboard from "./TimelineDashboard";
import PlaceholderDashboard from "./PlaceholderDashboard";

type ModuleType =
  | "timeline"
  | "text_upload"
  | "processes"
  | "screen"
  | "camera"
  | "microphone"
  | "keyboard"
  | "mouse"
  | "evdev_input"
  | "weather"
  | "hyprland";

// Define tab categories
type CategoryType = "data" | "input" | "system" | "environment";

interface TabDefinition {
  id: ModuleType;
  label: string;
  icon: React.ElementType;
  component: React.ReactNode;
  implemented: boolean;
  description: string;
  category: CategoryType;
}

interface CategoryDefinition {
  id: CategoryType;
  label: string;
  icon: React.ElementType;
  description: string;
}

export default function FeatureTabs() {
  const [activeTab, setActiveTab] = useState<ModuleType>("timeline");
  const [expandedCategory, setExpandedCategory] = useState<CategoryType>("data");
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);

  // Define categories
  const categories: CategoryDefinition[] = [
    {
      id: "data",
      label: "Data Capture",
      icon: LayoutGridIcon,
      description: "Manage data collection and storage"
    },
    {
      id: "input",
      label: "Input Devices",
      icon: InputIcon,
      description: "Monitor keyboard, mouse and input devices"
    },
    {
      id: "system",
      label: "System",
      icon: DesktopIcon,
      description: "Monitor system resources and processes"
    },
    {
      id: "environment",
      label: "Environment",
      icon: CloudIcon,
      description: "Track environmental factors"
    }
  ];

  // Define all available tabs
  const tabs: TabDefinition[] = [
    {
      id: "timeline",
      label: "Timeline",
      icon: Clock,
      component: <TimelineDashboard collectorId={null} />,
      implemented: true,
      description: "Browse your lifelog events by time",
      category: "data"
    },
    {
      id: "screen",
      label: "Screenshots",
      icon: CameraIcon,
      component: <ScreenDashboard collectorId={null} />,
      implemented: true,
      description: "Browse captured screenshots",
      category: "data"
    },
    {
      id: "text_upload",
      label: "Text Upload",
      icon: FileTextIcon,
      component: <TextUploadDashboard />,
      implemented: true,
      description: "Upload, search, and open files",
      category: "data"
    },
    {
      id: "camera",
      label: "Camera",
      icon: EyeIcon,
      component: <CameraDashboard />,
      implemented: true,
      description: "Manage camera recordings and snapshots",
      category: "data"
    },
    {
      id: "microphone",
      label: "Microphone",
      icon: MicIcon,
      component: <MicrophoneDashboard />,
      implemented: true,
      description: "Record and manage audio captures",
      category: "data"
    },
    {
      id: "keyboard",
      label: "Keyboard",
      icon: KeyboardIcon,
      component: <PlaceholderDashboard 
        title="Keyboard" 
        description="Track keyboard usage and statistics" 
        icon={KeyboardIcon} 
        moduleName="keyboard" 
      />,
      implemented: false,
      description: "Track keyboard usage and statistics",
      category: "input"
    },
    {
      id: "mouse",
      label: "Mouse",
      icon: MousePointerIcon,
      component: <PlaceholderDashboard 
        title="Mouse" 
        description="Monitor mouse movement and clicks" 
        icon={MousePointerIcon} 
        moduleName="mouse" 
      />,
      implemented: false,
      description: "Monitor mouse movement and clicks",
      category: "input"
    },
    {
      id: "evdev_input",
      label: "Input Logger",
      icon: InputIcon,
      component: <PlaceholderDashboard 
        title="Input Logger" 
        description="Track all input devices and events" 
        icon={InputIcon} 
        moduleName="evdev_input_logger" 
      />,
      implemented: false,
      description: "Track all input devices and events",
      category: "input"
    },
    {
      id: "processes",
      label: "Processes",
      icon: ActivityIcon,
      component: <ProcessesDashboard />,
      implemented: true,
      description: "View and monitor running processes",
      category: "system"
    },
    {
      id: "hyprland",
      label: "Hyprland",
      icon: PanelTopIcon,
      component: <PlaceholderDashboard 
        title="Hyprland" 
        description="Log Hyprland window manager events" 
        icon={PanelTopIcon} 
        moduleName="hyprland" 
      />,
      implemented: false,
      description: "Log Hyprland window manager events",
      category: "system"
    },
    {
      id: "weather",
      label: "Weather",
      icon: CloudIcon,
      component: <PlaceholderDashboard 
        title="Weather" 
        description="Monitor and log weather conditions" 
        icon={CloudIcon} 
        moduleName="weather" 
      />,
      implemented: false,
      description: "Monitor and log weather conditions",
      category: "environment"
    }
  ];

  // Get the active tab component
  const activeComponent = tabs.find(tab => tab.id === activeTab)?.component || null;
  const activeTabData = tabs.find(tab => tab.id === activeTab);
  const activeCategory = categories.find(cat => cat.id === activeTabData?.category);

  // Filter tabs by category
  const getTabsByCategory = (category: CategoryType) => 
    tabs.filter(tab => tab.category === category);

  // Toggle category
  const toggleCategory = (category: CategoryType) => {
    if (expandedCategory === category) {
      // Don't close - just keep it open
      return;
    }
    setExpandedCategory(category);
  };

  // Mobile menu toggle
  const toggleMobileMenu = () => {
    setIsMobileMenuOpen(!isMobileMenuOpen);
  };

  return (
    <div className="flex flex-col md:flex-row h-full">
      {/* Sidebar - Categories and Tabs */}
      <div className={cn(
        "md:w-64 bg-[#1A1E2E] border-r border-[#232B3D] md:flex flex-col",
        "transition-all duration-300 ease-in-out",
        "fixed md:static top-0 left-0 h-full z-30",
        isMobileMenuOpen ? "flex w-64" : "hidden"
      )}>
        {/* Mobile Close Button */}
        <button 
          className="md:hidden absolute top-4 right-4 p-2 text-[#9CA3AF] hover:text-[#F9FAFB]"
          onClick={toggleMobileMenu}
        >
          ×
        </button>

        <div className="p-4 md:p-0">
          <h2 className="text-[#F9FAFB] font-medium px-6 pt-6 pb-2">Modules</h2>
        </div>
        
        <div className="overflow-y-auto flex-1">
          {categories.map(category => (
            <div key={category.id} className="mb-1">
              <button
                onClick={() => toggleCategory(category.id)}
                className={cn(
                  "w-full flex items-center justify-between px-6 py-2 text-left transition-colors",
                  expandedCategory === category.id 
                    ? "bg-[#232B3D] text-[#F9FAFB]" 
                    : "text-[#9CA3AF] hover:text-[#F9FAFB] hover:bg-[#232B3D]/50"
                )}
              >
                <div className="flex items-center gap-2">
                  <category.icon className="w-4 h-4" />
                  <span className="font-medium">{category.label}</span>
                </div>
                <ChevronRightIcon className={cn(
                  "w-4 h-4 transition-transform",
                  expandedCategory === category.id && "rotate-90"
                )} />
              </button>
              
              {expandedCategory === category.id && (
                <div className="pl-4 pr-2 py-1">
                  {getTabsByCategory(category.id).map(tab => (
                    <button
                      key={tab.id}
                      onClick={() => {
                        setActiveTab(tab.id);
                        if (window.innerWidth < 768) {
                          setIsMobileMenuOpen(false);
                        }
                      }}
                      className={cn(
                        "w-full flex items-center gap-2 px-4 py-2 text-sm rounded-md my-1 transition-colors",
                        activeTab === tab.id
                          ? "bg-[#4C8BF5]/10 text-[#4C8BF5]"
                          : "text-[#9CA3AF] hover:text-[#F9FAFB] hover:bg-[#232B3D]",
                        !tab.implemented && "opacity-60"
                      )}
                      disabled={!tab.implemented}
                    >
                      <tab.icon className="w-4 h-4 flex-shrink-0" />
                      <span className="truncate">{tab.label}</span>
                      {!tab.implemented && (
                        <span className="ml-auto text-xs py-0.5 px-1.5 bg-[#232B3D] rounded text-[#9CA3AF]">
                          Soon
                        </span>
                      )}
                    </button>
                  ))}
                </div>
              )}
            </div>
          ))}
        </div>
      </div>

      {/* Main Content */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* Mobile Header */}
        <div className="md:hidden sticky top-0 z-10 border-b border-[#232B3D] bg-[#1A1E2E] p-4 flex items-center justify-between">
          <div className="flex items-center gap-2">
            {activeTabData && (
              <>
                <activeTabData.icon className="w-5 h-5 text-[#4C8BF5]" />
                <h1 className="font-medium text-[#F9FAFB]">{activeTabData.label}</h1>
              </>
            )}
          </div>
          <button 
            className="p-2 rounded-md text-[#9CA3AF] hover:text-[#F9FAFB] hover:bg-[#232B3D]"
            onClick={toggleMobileMenu}
          >
            <LayoutGridIcon className="w-5 h-5" />
          </button>
        </div>

        {/* Desktop Header */}
        <div className="hidden md:block border-b border-[#232B3D] bg-[#1A1E2E] p-4">
          <div className="flex items-start gap-3">
            {activeCategory && (
              <>
                <activeCategory.icon className="w-5 h-5 text-[#4C8BF5] mt-0.5" />
                <div className="text-left">
                  <h1 className="font-medium text-[#F9FAFB]">{activeCategory.label}</h1>
                  <p className="text-xs text-[#9CA3AF]">{activeCategory.description}</p>
                </div>
              </>
            )}
          </div>
        </div>

        {/* Tab Content */}
        <div className="flex-1 overflow-auto">
          {activeComponent}
        </div>
      </div>
    </div>
  );
} 
