import { Card } from './ui/card';
import { Navigation } from 'lucide-react';

interface LaneVisualizerProps {
  lanePosition: number; // -1 to 1, 0 is center
  steeringAngle: number; // -45 to 45 degrees
}

export function LaneVisualizer({ lanePosition, steeringAngle }: LaneVisualizerProps) {
  // Convert lane position to pixel offset (max 80px from center)
  const carOffset = lanePosition * 80;
  const isWellCentered = Math.abs(lanePosition) < 0.3;

  return (
    <Card className="p-6 bg-slate-800/50 border-slate-700">
      <div className="flex items-center gap-2 mb-4">
        <Navigation className="w-5 h-5 text-green-400" />
        <h3 className="text-white">Lane Position & Steering</h3>
      </div>

      {/* Lane Visualization */}
      <div className="mb-6">
        <div className="relative h-40 bg-slate-900 rounded-lg overflow-hidden">
          {/* Road markings */}
          <div className="absolute inset-0 flex justify-between px-8">
            {/* Left lane marker */}
            <div className="w-1 h-full bg-yellow-400"></div>
            
            {/* Center dashed line */}
            <div className="flex flex-col justify-around">
              {[...Array(8)].map((_, i) => (
                <div key={i} className="w-1 h-4 bg-white"></div>
              ))}
            </div>
            
            {/* Right lane marker */}
            <div className="w-1 h-full bg-yellow-400"></div>
          </div>

          {/* Vehicle representation */}
          <div 
            className="absolute bottom-4 left-1/2 transition-all duration-300"
            style={{ 
              transform: `translateX(calc(-50% + ${carOffset}px))`,
            }}
          >
            <div 
              className={`w-12 h-20 rounded-lg ${isWellCentered ? 'bg-green-500' : 'bg-orange-500'} border-2 border-white shadow-lg transition-colors duration-300`}
              style={{
                transform: `rotate(${steeringAngle * 0.5}deg)`,
              }}
            >
              {/* Car details */}
              <div className="h-full flex flex-col justify-between p-1">
                <div className="h-2 bg-white/30 rounded"></div>
                <div className="h-2 bg-white/30 rounded"></div>
              </div>
            </div>
          </div>
        </div>

        {/* Position Indicator */}
        <div className="mt-4 text-center">
          <p className="text-slate-400 text-sm">Lane Position</p>
          <p className={`text-xl ${isWellCentered ? 'text-green-400' : 'text-orange-400'}`}>
            {lanePosition > 0 ? 'Right ' : lanePosition < 0 ? 'Left ' : 'Center '}
            {Math.abs(lanePosition * 100).toFixed(0)}%
          </p>
        </div>
      </div>
    </Card>
  );
}