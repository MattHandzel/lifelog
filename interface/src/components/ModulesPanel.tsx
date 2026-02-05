import React from 'react';
import { useState } from 'react';
import { NavLink, useLocation } from 'react-router-dom';
import { 
  Camera, 
  Mic, 
  Monitor, 
  Layers,
  Laptop,
  GanttChart,
  Cloud,
  ChevronRight,
  Keyboard,
  Mouse,
  Activity,
  Thermometer,
  Wifi,
  MapPin,
  Upload,
  Layout,
  File,
  HardDrive,
  Cpu,
  FileText
} from 'lucide-react';

type ModuleItemProps = {
  to?: string;
  icon: React.ReactNode;
  label: string;
  isExpanded?: boolean;
  onClick?: () => void;
  children?: React.ReactNode;
  coming?: boolean;
  disabled?: boolean;
};

const ModuleItem = ({ 
  to, 
  icon, 
  label, 
  isExpanded, 
  onClick, 
  children,
  coming,
  disabled
}: ModuleItemProps) => {
  // If it's a navigation item with a link
  if (to && !disabled) {
    return (
      <NavLink 
        to={to} 
        className={({ isActive }) => 
          `flex items-center justify-between px-4 py-3 rounded-lg transition-all duration-200
          ${isActive ? 'bg-[#232B3D] text-[#4C8BF5]' : 'text-[#9CA3AF] hover:text-white hover:bg-[#232B3D]'}`
        }
      >
        <div className="flex items-center gap-3">
          {icon}
          <span>{label}</span>
        </div>
        {coming && (
          <span className="px-2 py-1 text-xs bg-[#232B3D] text-[#9CA3AF] rounded">Soon</span>
        )}
      </NavLink>
    );
  }
  
  // If it's a disabled item
  if (disabled) {
    return (
      <div className="flex items-center justify-between px-4 py-3 rounded-lg text-gray-600 cursor-not-allowed">
        <div className="flex items-center gap-3">
          {icon}
          <span>{label}</span>
        </div>
        <span className="px-2 py-1 text-xs bg-[#232B3D] text-[#9CA3AF] rounded">
          {coming ? 'Soon' : 'Unavailable'}
        </span>
      </div>
    );
  }
  
  // If it's a section header with children
  return (
    <div>
      <button 
        onClick={onClick}
        className="flex items-center justify-between w-full px-4 py-3 text-[#9CA3AF] hover:text-white transition-colors rounded-lg"
      >
        <div className="flex items-center gap-3">
          {icon}
          <span>{label}</span>
        </div>
        <ChevronRight 
          size={16} 
          className={`transition-transform duration-200 ${isExpanded ? 'rotate-90' : ''}`} 
        />
      </button>
      
      {isExpanded && children && (
        <div className="ml-4 mt-1 space-y-1 border-l border-[#232B3D] pl-3">
          {children}
        </div>
      )}
    </div>
  );
};

const ModulesPanel: React.FC = () => {
  const location = useLocation();
  const [expandedSections, setExpandedSections] = useState<Record<string, boolean>>({
    'data': true,
    'input': false,
    'system': false,
    'environment': false
  });
  
  const toggleSection = (section: string) => {
    setExpandedSections(prev => ({
      ...prev,
      [section]: !prev[section]
    }));
  };

  return (
    <div className="w-64 bg-[#141E33] h-full flex flex-col border-r border-[#232B3D]">
      <div className="p-4 border-b border-[#232B3D]">
        <h2 className="text-[#9CA3AF] font-medium">Modules</h2>
      </div>
      
      <nav className="flex-1 px-2 py-4 space-y-1 overflow-y-auto">
        <ModuleItem 
          icon={<Layers size={18} />} 
          label="Data Capture" 
          isExpanded={expandedSections.data}
          onClick={() => toggleSection('data')}
        >
          <ModuleItem 
            to="/text" 
            icon={<FileText size={16} />} 
            label="Text Upload" 
          />
          <ModuleItem 
            to="/screenshots" 
            icon={<Monitor size={16} />} 
            label="Screenshots" 
          />
          <ModuleItem 
            to="/camera" 
            icon={<Camera size={16} />} 
            label="Camera" 
          />
          <ModuleItem 
            to="/microphone" 
            icon={<Mic size={16} />} 
            label="Microphone" 
            coming={true}
          />
        </ModuleItem>
        
        <ModuleItem 
          icon={<Laptop size={18} />} 
          label="Input Devices" 
          isExpanded={expandedSections.input}
          onClick={() => toggleSection('input')}
        >
          <ModuleItem 
            icon={<Keyboard size={16} />} 
            label="Keyboard" 
            disabled={true}
            coming={true}
          />
          <ModuleItem 
            icon={<Mouse size={16} />} 
            label="Mouse" 
            disabled={true}
            coming={true}
          />
        </ModuleItem>
        
        <ModuleItem 
          icon={<GanttChart size={18} />} 
          label="System" 
          isExpanded={expandedSections.system}
          onClick={() => toggleSection('system')}
        >
          <ModuleItem 
            to="/processes" 
            icon={<Activity size={16} />} 
            label="Processes" 
          />
        </ModuleItem>
        
        <ModuleItem 
          icon={<Cloud size={18} />} 
          label="Environment" 
          isExpanded={expandedSections.environment}
          onClick={() => toggleSection('environment')}
        >
          <ModuleItem 
            icon={<Thermometer size={16} />} 
            label="Weather" 
            disabled={true}
            coming={true}
          />
        </ModuleItem>
      </nav>
    </div>
  );
}

export default ModulesPanel; 