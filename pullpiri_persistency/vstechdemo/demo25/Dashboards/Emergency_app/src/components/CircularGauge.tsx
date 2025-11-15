import { Badge } from './ui/badge';

interface CircularGaugeProps {
  value: number;
  max: number;
  unit: string;
  color?: 'blue' | 'green' | 'yellow' | 'orange' | 'red';
  label?: string;
  description?: string;
}

export function CircularGauge({ value, max, unit, color = 'blue', label, description }: CircularGaugeProps) {
  const safeValue = value ?? 0;
  const safeMax = max ?? 100;
  const percentage = Math.min((safeValue / safeMax) * 100, 100);
  const circumference = 2 * Math.PI * 70;
  const strokeDashoffset = circumference - (percentage / 100) * circumference;

  const colorMap = {
    blue: '#3b82f6',
    green: '#22c55e',
    yellow: '#f59e0b',
    orange: '#f59e0b',
    red: '#ef4444'
  };

  const strokeColor = colorMap[color];

  // Determine status badge
  const getStatusBadge = () => {
    if (label === 'Vehicle Speed') {
      if (safeValue > 40) return { text: 'High', color: 'bg-orange-500' };
      if (safeValue > 25) return { text: 'Medium', color: 'bg-yellow-500' };
      return { text: 'Safe', color: 'bg-green-600' };
    }
    if (label === 'Brake Force') {
      if (safeValue > 70) return { text: 'Max', color: 'bg-red-600' };
      if (safeValue > 40) return { text: 'Active', color: 'bg-orange-500' };
      return { text: 'Normal', color: 'bg-green-600' };
    }
    return null;
  };

  const statusBadge = getStatusBadge();

  return (
    <div className="flex flex-col items-center">
      <div className="relative w-48 h-48 mb-3">
        <svg className="transform -rotate-90 w-48 h-48">
          {/* Background circle */}
          <circle
            cx="96"
            cy="96"
            r="70"
            stroke="#1e293b"
            strokeWidth="16"
            fill="none"
          />
          {/* Progress circle */}
          <circle
            cx="96"
            cy="96"
            r="70"
            stroke={strokeColor}
            strokeWidth="16"
            fill="none"
            strokeDasharray={circumference}
            strokeDashoffset={strokeDashoffset}
            strokeLinecap="round"
            className="transition-all duration-300"
          />
        </svg>
        {/* Center text */}
        <div className="absolute inset-0 flex flex-col items-center justify-center">
          <span className="text-white">{Math.round(safeValue)}</span>
          <span className="text-slate-400 text-sm">{unit || ''}</span>
        </div>
      </div>
      
      {/* Status badge */}
      {statusBadge && (
        <Badge className={`${statusBadge.color} mb-2`}>
          {statusBadge.text}
        </Badge>
      )}
      
      {/* Description */}
      {description && (
        <p className="text-slate-400 text-sm text-center max-w-xs">
          {description}
        </p>
      )}
    </div>
  );
}