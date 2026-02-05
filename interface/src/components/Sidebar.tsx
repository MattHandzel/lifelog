import { useEffect } from 'react';
import { NavLink } from 'react-router-dom';
import { 
  LayoutDashboard,
  Settings
} from 'lucide-react';

export function Sidebar() {
  return (
    <aside className="w-[15%] bg-[#121825] h-full flex flex-col border-r border-[#232B3D]">
      <div className="p-4 border-b border-[#232B3D]">
        <h1 className="text-2xl font-bold text-[#4C8BF5]">LifeLog</h1>
      </div>
      
      <div className="flex-1 p-2">
        <NavLink 
          to="/dashboard" 
          className={({ isActive }) => 
            `flex items-center gap-3 px-4 py-3 rounded-lg transition-all duration-200
            ${isActive ? 'bg-[#232B3D] text-[#4C8BF5]' : 'text-[#9CA3AF] hover:text-white hover:bg-[#232B3D]'}`
          }
        >
          <LayoutDashboard size={18} />
          <span>Dashboard</span>
        </NavLink>
      </div>
      
      <div className="mt-auto p-2">
        <NavLink 
          to="/settings" 
          className={({ isActive }) => 
            `flex items-center gap-3 px-4 py-3 rounded-lg transition-all duration-200
            ${isActive ? 'bg-[#232B3D] text-[#4C8BF5]' : 'text-[#9CA3AF] hover:text-white hover:bg-[#232B3D]'}`
          }
        >
          <Settings size={18} />
          <span>Settings</span>
        </NavLink>
      </div>
    </aside>
  );
}

export default Sidebar; 