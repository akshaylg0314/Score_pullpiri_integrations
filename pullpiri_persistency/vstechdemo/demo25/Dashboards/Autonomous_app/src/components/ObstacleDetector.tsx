import { Card } from './ui/card';
import { Badge } from './ui/badge';
import { AlertTriangle } from 'lucide-react';

interface ObstacleDetectorProps {
  obstacleDetected: boolean;
  obstacleDistance: number;
}

export function ObstacleDetector({ obstacleDetected, obstacleDistance }: ObstacleDetectorProps) {
  const safeDistance = obstacleDistance ?? 100;
  const getDistanceStatus = () => {
    if (!obstacleDetected) return 'safe';
    if (obstacleDistance < 15) return 'critical';
    if (obstacleDistance < 30) return 'warning';
    return 'caution';
  };

  const getDistanceColor = () => {
    const status = getDistanceStatus();
    switch (status) {
      case 'critical':
        return 'text-red-400';
      case 'warning':
        return 'text-orange-400';
      case 'caution':
        return 'text-yellow-400';
      default:
        return 'text-green-400';
    }
  };

  const getRadarColor = () => {
    const status = getDistanceStatus();
    switch (status) {
      case 'critical':
        return '#ef4444';
      case 'warning':
        return '#f59e0b';
      case 'caution':
        return '#eab308';
      default:
        return '#3b82f6';
    }
  };

  return (
    <Card className="p-6 bg-slate-800/50 border-slate-700">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <AlertTriangle className="w-5 h-5 text-yellow-400" />
          <h3 className="text-white">Obstacle Detection</h3>
        </div>
        <Badge variant={obstacleDetected ? "destructive" : "secondary"}>
          {obstacleDetected ? 'Obstacle Detected' : 'Clear'}
        </Badge>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Status Info */}
        <div className="space-y-4">
          {obstacleDetected ? (
            <div className={`p-4 bg-slate-900/50 rounded-lg border ${
              getDistanceStatus() === 'critical' ? 'border-red-600/50' :
              getDistanceStatus() === 'warning' ? 'border-orange-600/50' :
              'border-yellow-600/50'
            }`}>
              <div className="flex items-center justify-between mb-2">
                <span className="text-slate-400 text-sm">Distance</span>
                <span className={`text-2xl ${getDistanceColor()}`}>
                  {safeDistance.toFixed(3)}m
                </span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-slate-400 text-sm">Status</span>
                <span className={`${getDistanceColor()} capitalize`}>
                  {getDistanceStatus()}
                </span>
              </div>
            </div>
          ) : (
            <div className="text-center py-8">
              <div className="w-16 h-16 mx-auto mb-3 rounded-full bg-green-950/50 flex items-center justify-center">
                <div className="w-3 h-3 rounded-full bg-green-500"></div>
              </div>
              <p className="text-slate-400">No obstacles detected</p>
              <p className="text-slate-500 text-sm">Clear path ahead</p>
            </div>
          )}

          {/* Safety Thresholds */}
          <div className="pt-4 border-t border-slate-700">
            <p className="text-slate-400 text-sm mb-3">Safety Thresholds</p>
            <div className="space-y-2">
              <div className="flex items-center gap-2">
                <div className="w-3 h-3 rounded-full bg-red-500"></div>
                <span className="text-slate-400 text-sm">Critical: {'<'} 15m</span>
              </div>
              <div className="flex items-center gap-2">
                <div className="w-3 h-3 rounded-full bg-orange-500"></div>
                <span className="text-slate-400 text-sm">Warning: 15-30m</span>
              </div>
              <div className="flex items-center gap-2">
                <div className="w-3 h-3 rounded-full bg-yellow-500"></div>
                <span className="text-slate-400 text-sm">Caution: 30-45m</span>
              </div>
              <div className="flex items-center gap-2">
                <div className="w-3 h-3 rounded-full bg-green-500"></div>
                <span className="text-slate-400 text-sm">Safe: {'>'} 45m or clear</span>
              </div>
            </div>
          </div>
        </div>

        {/* Radar Visualization */}
        <div>
          <div className="relative w-full aspect-square max-w-xs mx-auto">
            <svg className="w-full h-full" viewBox="0 0 200 200">
              {/* Radar circles */}
              <circle cx="100" cy="100" r="80" fill="none" stroke="#334155" strokeWidth="1" />
              <circle cx="100" cy="100" r="60" fill="none" stroke="#334155" strokeWidth="1" />
              <circle cx="100" cy="100" r="40" fill="none" stroke="#334155" strokeWidth="1" />
              <circle cx="100" cy="100" r="20" fill="none" stroke="#334155" strokeWidth="1" />
              
              {/* Cross lines */}
              <line x1="100" y1="20" x2="100" y2="180" stroke="#334155" strokeWidth="1" />
              <line x1="20" y1="100" x2="180" y2="100" stroke="#334155" strokeWidth="1" />
              
              {/* Vehicle in center */}
              <circle cx="100" cy="100" r="5" fill="#3b82f6" />
              
              {/* Obstacle */}
              {obstacleDetected && (
                <>
                  {/* Obstacle point */}
                  <circle
                    cx="100"
                    cy={100 - (obstacleDistance / 50) * 80}
                    r="6"
                    fill={getRadarColor()}
                    className="animate-pulse"
                  />
                  {/* Distance line */}
                  <line
                    x1="100"
                    y1="100"
                    x2="100"
                    y2={100 - (obstacleDistance / 50) * 80}
                    stroke={getRadarColor()}
                    strokeWidth="1"
                    strokeDasharray="2,2"
                  />
                </>
              )}
            </svg>
            
            {/* Distance labels */}
            <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
              <div className="relative w-full h-full">
                <span className="absolute top-4 left-1/2 -translate-x-1/2 text-slate-500 text-xs">50m</span>
                <span className="absolute bottom-4 left-1/2 -translate-x-1/2 text-slate-500 text-xs">Rear</span>
                <span className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-500 text-xs">L</span>
                <span className="absolute right-4 top-1/2 -translate-y-1/2 text-slate-500 text-xs">R</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Card>
  );
}