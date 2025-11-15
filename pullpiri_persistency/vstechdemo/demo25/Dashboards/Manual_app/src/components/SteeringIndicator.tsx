interface SteeringIndicatorProps {
  angle: number;
}

export default function SteeringIndicator({ angle }: SteeringIndicatorProps) {
  return (
    <div className="relative w-48 h-48">
      <svg className="w-full h-full" viewBox="0 0 200 200">
        {/* Outer rim */}
        <circle
          cx="100"
          cy="100"
          r="80"
          fill="none"
          stroke="#334155"
          strokeWidth="8"
        />
        
        {/* Inner circle */}
        <circle
          cx="100"
          cy="100"
          r="60"
          fill="#1e293b"
          stroke="#475569"
          strokeWidth="2"
        />

        {/* Steering wheel spokes - rotates with angle */}
        <g 
          transform={`rotate(${angle} 100 100)`}
          className="transition-all duration-300"
        >
          {/* Horizontal spoke */}
          <line
            x1="40"
            y1="100"
            x2="160"
            y2="100"
            stroke="#3b82f6"
            strokeWidth="4"
            strokeLinecap="round"
          />
          {/* Vertical spoke */}
          <line
            x1="100"
            y1="40"
            x2="100"
            y2="160"
            stroke="#3b82f6"
            strokeWidth="4"
            strokeLinecap="round"
          />
          
          {/* Top indicator */}
          <circle
            cx="100"
            cy="40"
            r="8"
            fill="#ef4444"
          />
        </g>

        {/* Center hub */}
        <circle
          cx="100"
          cy="100"
          r="15"
          fill="#334155"
          stroke="#475569"
          strokeWidth="2"
        />
      </svg>

      {/* Angle indicators */}
      <div className="absolute top-2 left-1/2 transform -translate-x-1/2 text-slate-400 text-sm">
        Left
      </div>
      <div className="absolute bottom-2 left-1/2 transform -translate-x-1/2 text-slate-400 text-sm">
        Right
      </div>
    </div>
  );
}
