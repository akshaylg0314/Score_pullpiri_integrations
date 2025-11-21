import { Navigation } from 'lucide-react';

interface SteeringWheelProps {
  angle: number;
}

export function SteeringWheel({ angle }: SteeringWheelProps) {
  // Calculate the position of the red indicator dot based on angle
  // angle is in degrees, -90 to +90 typically
  // We'll map this to a position around the circle
  const safeAngle = angle ?? 0;
  const angleRad = (safeAngle * Math.PI) / 180;
  const indicatorRadius = 65;
  const indicatorX = 100 + indicatorRadius * Math.sin(angleRad);
  const indicatorY = 100 - indicatorRadius * Math.cos(angleRad);

  return (
    <div className="flex flex-col items-center">
      <div className="relative w-48 h-48 mb-3">
        <svg viewBox="0 0 200 200" className="w-48 h-48">
          {/* Outer circle background */}
          <circle 
            cx="100" 
            cy="100" 
            r="75" 
            fill="none" 
            stroke="#1e293b" 
            strokeWidth="8"
          />
          
          {/* Inner compass circle */}
          <circle 
            cx="100" 
            cy="100" 
            r="60" 
            fill="none" 
            stroke="#334155" 
            strokeWidth="2"
          />
          
          {/* Cross indicator (rotates with steering) */}
          <g transform={`rotate(${safeAngle} 100 100)`} className="transition-all duration-300">
            {/* Vertical line */}
            <line 
              x1="100" 
              y1="50" 
              x2="100" 
              y2="150" 
              stroke="#3b82f6" 
              strokeWidth="3"
            />
            {/* Horizontal line */}
            <line 
              x1="50" 
              y1="100" 
              x2="150" 
              y2="100" 
              stroke="#3b82f6" 
              strokeWidth="3"
            />
          </g>
          
          {/* Direction labels */}
          <text 
            x="100" 
            y="30" 
            textAnchor="middle" 
            fill="#64748b" 
            fontSize="12"
          >
            Left
          </text>
          <text 
            x="100" 
            y="178" 
            textAnchor="middle" 
            fill="#64748b" 
            fontSize="12"
          >
            Right
          </text>
          
          {/* Red indicator dot showing current angle */}
          <circle 
            cx={indicatorX} 
            cy={indicatorY} 
            r="6" 
            fill="#ef4444"
            className="transition-all duration-300"
          />
        </svg>
      </div>
      
      {/* Current angle display */}
      <div className="text-center mb-2">
        <p className="text-slate-400 text-sm mb-1">Current Angle</p>
        <p className="text-white">{safeAngle.toFixed(1)}°</p>
      </div>
      
      {/* Description */}
      <p className="text-slate-400 text-sm text-center max-w-xs">
        Adjusts steering to navigate turns and curves safely
      </p>
    </div>
  );
}