import { Card } from './ui/card';
import { Badge } from './ui/badge';
import { Gauge } from 'lucide-react';

interface SpeedGaugeProps {
  currentSpeed: number;
}

export function SpeedGauge({ currentSpeed }: SpeedGaugeProps) {
  const safeSpeed = currentSpeed ?? 0;
  const maxSpeed = 120;
  const speedPercentage = Math.min((safeSpeed / maxSpeed) * 100, 100);
  
  // Determine speed status
  const getSpeedStatus = () => {
    if (currentSpeed > 90) return { label: 'High', color: 'bg-red-600' };
    if (currentSpeed > 60) return { label: 'Normal', color: 'bg-green-600' };
    return { label: 'Low', color: 'bg-blue-600' };
  };

  const status = getSpeedStatus();
  
  return (
    <Card className="p-6 bg-slate-800/50 border-slate-700">
      <div className="flex items-center gap-2 mb-4">
        <Gauge className="w-5 h-5 text-blue-400" />
        <h3 className="text-white">Vehicle Speed</h3>
      </div>
      
      <div className="relative">
        {/* Circular Gauge */}
        <div className="relative w-48 h-48 mx-auto">
          <svg className="transform -rotate-90" width="192" height="192">
            {/* Background circle */}
            <circle
              cx="96"
              cy="96"
              r="80"
              stroke="#1e293b"
              strokeWidth="16"
              fill="none"
            />
            {/* Progress circle - Orange/Yellow gradient */}
            <circle
              cx="96"
              cy="96"
              r="80"
              stroke="#f59e0b"
              strokeWidth="16"
              fill="none"
              strokeDasharray={`${(speedPercentage / 100) * 502.65} 502.65`}
              strokeLinecap="round"
              className="transition-all duration-300"
            />
          </svg>
          
          {/* Center content */}
          <div className="absolute inset-0 flex flex-col items-center justify-center">
            <span className="text-5xl text-white">
              {Math.round(safeSpeed)}
            </span>
            <span className="text-slate-400">km/h</span>
          </div>
        </div>

        {/* Speed Status Badge */}
        <div className="mt-4 flex justify-center">
          <Badge className={status.color}>{status.label}</Badge>
        </div>

        {/* Description */}
        <div className="mt-4 text-center">
          <p className="text-slate-400 text-sm">
            Compliance with speed limits and safe driving conditions
          </p>
        </div>
      </div>
    </Card>
  );
}
