import * as React from "react";
import { cn } from "../../lib/utils";

interface ProgressProps extends React.HTMLAttributes<HTMLDivElement> {
  value: number;
  max?: number;
  showValue?: boolean;
  size?: "sm" | "md" | "lg";
  variant?: "default" | "success" | "info" | "warning" | "danger";
}

const sizeClasses = {
  sm: "h-1",
  md: "h-2",
  lg: "h-3",
};

const variantClasses = {
  default: "bg-primary",
  success: "bg-green-500",
  info: "bg-blue-500",
  warning: "bg-yellow-500",
  danger: "bg-red-500",
};

export function Progress({
  value,
  max = 100,
  showValue = false,
  size = "md",
  variant = "default",
  className,
  ...props
}: ProgressProps) {
  const percentage = Math.min(100, Math.max(0, (value / max) * 100));

  return (
    <div className={cn("w-full", className)} {...props}>
      <div
        className="flex items-center gap-2"
      >
        <div className={cn("w-full overflow-hidden rounded-full bg-gray-200 dark:bg-gray-700", sizeClasses[size])}>
          <div
            className={cn("transition-all duration-300 ease-in-out", variantClasses[variant])}
            style={{ width: `${percentage}%` }}
          />
        </div>
        
        {showValue && (
          <span className="text-xs font-medium text-gray-700 dark:text-gray-300 min-w-[40px] text-right">
            {Math.round(percentage)}%
          </span>
        )}
      </div>
    </div>
  );
} 