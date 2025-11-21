interface ObstacleRadarProps {
  obstacleDetected: boolean;
  obstacleDistance: number;
}

export function ObstacleRadar({ obstacleDetected, obstacleDistance }: ObstacleRadarProps) {
  // Calculate obstacle position based on distance (closer = nearer to center)
  const maxDistance = 50; // meters
  const normalizedDistance = Math.min(obstacleDistance / maxDistance, 1);
  const obstacleRadius = 100 - (normalizedDistance * 70); // Position from center

  // Determine obstacle color based on distance
  const getObstacleColor = () => {
    if (obstacleDistance < 2) return '#ef4444'; // red
    if (obstacleDistance < 5) return '#f59e0b'; // orange
    if (obstacleDistance < 10) return '#eab308'; // yellow
    return '#22c55e'; // green
  };

  return (
    <div className="flex items-center justify-center">
      <svg viewBox="0 0 200 200" className="w-64 h-64">
        {/* Radar circles (range indicators) */}
        <circle cx="100" cy="100" r="80" fill="none" stroke="#334155" strokeWidth="1" />
        <circle cx="100" cy="100" r="60" fill="none" stroke="#334155" strokeWidth="1" />
        <circle cx="100" cy="100" r="40" fill="none" stroke="#334155" strokeWidth="1" />
        <circle cx="100" cy="100" r="20" fill="none" stroke="#334155" strokeWidth="1" />
        
        {/* Grid lines */}
        <line x1="100" y1="20" x2="100" y2="180" stroke="#334155" strokeWidth="1" />
        <line x1="20" y1="100" x2="180" y2="100" stroke="#334155" strokeWidth="1" />
        
        {/* Scanning effect (optional pulsing ring) */}
        <circle 
          cx="100" 
          cy="100" 
          r="80" 
          fill="none" 
          stroke="#3b82f6" 
          strokeWidth="2" 
          opacity="0.3"
          className="animate-pulse"
        />
        
        {/* Vehicle (center) */}
        <rect 
          x="90" 
          y="90" 
          width="20" 
          height="20" 
          fill="#3b82f6" 
          rx="2"
        />
        <polygon 
          points="100,85 95,90 105,90" 
          fill="#60a5fa"
        />
        
        {/* Obstacle (if detected) */}
        {obstacleDetected && (
          <>
            <circle 
              cx="100" 
              cy={100 - obstacleRadius} 
              r="8" 
              fill={getObstacleColor()}
              className="animate-pulse"
            />
            <circle 
              cx="100" 
              cy={100 - obstacleRadius} 
              r="12" 
              fill="none" 
              stroke={getObstacleColor()}
              strokeWidth="2"
              opacity="0.5"
            />
            
            {/* Distance line */}
            <line 
              x1="100" 
              y1="100" 
              x2="100" 
              y2={100 - obstacleRadius} 
              stroke={getObstacleColor()}
              strokeWidth="1" 
              strokeDasharray="2,2"
              opacity="0.5"
            />
          </>
        )}
        
        {/* Range labels */}
        <text x="100" y="25" textAnchor="middle" fill="#64748b" fontSize="10">
          50m
        </text>
        <text x="100" y="185" textAnchor="middle" fill="#64748b" fontSize="10">
          Rear
        </text>
        <text x="15" y="105" textAnchor="middle" fill="#64748b" fontSize="10">
          Left
        </text>
        <text x="185" y="105" textAnchor="middle" fill="#64748b" fontSize="10">
          Right
        </text>
      </svg>
    </div>
  );
}
