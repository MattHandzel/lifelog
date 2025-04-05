import { LucideIcon } from "lucide-react";
import { Button } from "./ui/button";

interface PlaceholderDashboardProps {
  title: string;
  description: string;
  icon: LucideIcon;
  moduleName: string;
}

export default function PlaceholderDashboard({
  title,
  description,
  icon: Icon,
  moduleName
}: PlaceholderDashboardProps) {
  return (
    <div className="p-6 md:p-8 space-y-6">
      <div className="mb-8">
        <div className="flex items-center gap-3 mb-2">
          <Icon className="w-8 h-8 text-[#4C8BF5]" />
          <h1 className="title">{title}</h1>
        </div>
        <p className="subtitle">{description}</p>
      </div>

      <div className="card">
        <div className="p-12 flex flex-col items-center justify-center text-center">
          <div className="w-24 h-24 rounded-full flex items-center justify-center bg-[#1C2233] mb-6">
            <Icon className="w-12 h-12 text-[#4C8BF5] animate-float" />
          </div>
          
          <h2 className="text-xl font-semibold text-[#F9FAFB] mb-2">
            {title} Module
          </h2>
          
          <p className="text-[#9CA3AF] max-w-md mb-8">
            This feature is under development and will be available soon. The {moduleName} module 
            exists in the backend but hasn't been implemented in the UI yet.
          </p>
          
          <div className="flex flex-col sm:flex-row gap-4">
            <Button className="btn-secondary">
              Learn More
            </Button>
            <Button className="btn-primary">
              Get Notified When Available
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
} 