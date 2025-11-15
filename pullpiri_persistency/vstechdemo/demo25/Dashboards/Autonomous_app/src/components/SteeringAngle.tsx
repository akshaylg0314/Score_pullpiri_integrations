import { Card } from './ui/card';
import { Navigation2 } from 'lucide-react';

interface SteeringAngleProps {
  steeringAngle: number; // degrees
}

export function SteeringAngle({ steeringAngle }: SteeringAngleProps) {
  // Convert angle to position on circle (in degrees)
  // -45 to +45 degrees, where negative is left, positive is right
  const safeAngle = steeringAngle ?? 0;
  const angleInRadians = (safeAngle * Math.PI) / 180;
  
  // Calculate position of the red dot on the circle
  const radius = 70; // radius of the circle
  const dotX = 96 + radius * Math.sin(angleInRadians);
  const dotY = 96 - radius * Math.cos(angleInRadians);

  return (
    <Card className="p-6 bg-slate-800/50 border-slate-700">
      <div className="flex items-center gap-2 mb-4">
        <Navigation2 className="w-5 h-5 text-blue-400" />
        <h3 className="text-white">Steering Angle</h3>
      </div>
      
      <div className="relative">
        {/* Circular Compass */}
        <div className="relative w-48 h-48 mx-auto">
          <svg width="192" height="192">
            {/* Outer circle */}
            <circle
              cx="96"
              cy="96"
              r="80"
              stroke="#334155"
              strokeWidth="2"
              fill="none"
            />
            
            {/* Inner circles */}
            <circle
              cx="96"
              cy="96"
              r="70"
              stroke="#1e293b"
              strokeWidth="1"
              fill="none"
            />
            <circle
              cx="96"
              cy="96"
              r="50"
              stroke="#1e293b"
              strokeWidth="1"
              fill="none"
            />
            
            {/* Cross lines (Blue) */}
            <line x1="96" y1="26" x2="96" y2="166" stroke="#3b82f6" strokeWidth="2" />
            <line x1="26" y1="96" x2="166" y2="96" stroke="#3b82f6" strokeWidth="2" />
            
            {/* Center circle */}
            <circle cx="96" cy="96" r="8" fill="#1e293b" />
            
            {/* Red indicator dot */}
            <circle
              cx={dotX}
              cy={dotY}
              r="6"
              fill="#ef4444"
              className="transition-all duration-300"
            />
          </svg>
          
          {/* Direction labels */}
          <div className="absolute top-2 left-1/2 -translate-x-1/2">
            <span className="text-slate-400 text-sm">Left</span>
          </div>
          <div className="absolute bottom-2 left-1/2 -translate-x-1/2">
            <span className="text-slate-400 text-sm">Right</span>
          </div>
        </div>

        {/* Current Angle Display */}
        <div className="mt-4 text-center">
          <p className="text-slate-400 text-sm mb-1">Current Angle</p>
          <p className="text-white text-xl">{safeAngle.toFixed(1)}°</p>
        </div>

        {/* Description */}
        <div className="mt-4 text-center">
          <p className="text-slate-400 text-sm">
            Adjusts steering to navigate turns and curves safely
          </p>
        </div>
      </div>
    </Card>
  );
}
