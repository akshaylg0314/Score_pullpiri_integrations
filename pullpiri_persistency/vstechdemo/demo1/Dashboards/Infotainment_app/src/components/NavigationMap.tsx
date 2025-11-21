import { Card } from './ui/card';
import { Navigation, MapPin, Clock } from 'lucide-react';

export function NavigationMap() {
  return (
    <Card className="p-6 bg-slate-800/50 border-slate-700">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <Navigation className="w-5 h-5 text-blue-400" />
          <h3 className="text-white">Navigation</h3>
        </div>
        <div className="flex items-center gap-2">
          <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse" />
          <span className="text-green-400 text-sm">Active Route</span>
        </div>
      </div>

      {/* Mock Map Display */}
      <div className="relative bg-slate-900 rounded-lg overflow-hidden mb-4" style={{ height: '280px' }}>
        {/* Grid Pattern Background */}
        <svg className="absolute inset-0 w-full h-full" style={{ opacity: 0.3 }}>
          <defs>
            <pattern id="grid" width="40" height="40" patternUnits="userSpaceOnUse">
              <path d="M 40 0 L 0 0 0 40" fill="none" stroke="#334155" strokeWidth="1"/>
            </pattern>
          </defs>
          <rect width="100%" height="100%" fill="url(#grid)" />
        </svg>

        {/* Route Line */}
        <svg className="absolute inset-0 w-full h-full">
          <path
            d="M 50 250 Q 150 200, 250 180 T 450 140"
            fill="none"
            stroke="#3b82f6"
            strokeWidth="4"
            strokeLinecap="round"
            opacity="0.8"
          />
          {/* Route dots */}
          <circle cx="50" cy="250" r="6" fill="#10b981" />
          <circle cx="250" cy="180" r="4" fill="#3b82f6" />
          <circle cx="450" cy="140" r="6" fill="#ef4444" />
        </svg>

        {/* Current Position Indicator */}
        <div className="absolute left-12 bottom-12 flex items-center gap-2 bg-blue-600 px-3 py-1.5 rounded-full">
          <div className="w-2 h-2 rounded-full bg-white animate-pulse" />
          <span className="text-white text-sm">You are here</span>
        </div>

        {/* Destination Marker */}
        <div className="absolute right-12 top-12 flex items-center gap-2 bg-red-600 px-3 py-1.5 rounded-full">
          <MapPin className="w-4 h-4 text-white" />
          <span className="text-white text-sm">Destination</span>
        </div>
      </div>

      {/* Route Information */}
      <div className="grid grid-cols-3 gap-4">
        <div className="bg-slate-900/50 p-3 rounded-lg">
          <p className="text-slate-400 text-sm mb-1">Distance</p>
          <p className="text-white">12.4 km</p>
        </div>
        <div className="bg-slate-900/50 p-3 rounded-lg">
          <p className="text-slate-400 text-sm mb-1">ETA</p>
          <div className="flex items-center gap-1">
            <Clock className="w-4 h-4 text-blue-400" />
            <p className="text-white">18 min</p>
          </div>
        </div>
        <div className="bg-slate-900/50 p-3 rounded-lg">
          <p className="text-slate-400 text-sm mb-1">Speed</p>
          <p className="text-white">65 km/h</p>
        </div>
      </div>

      {/* Next Turn */}
      <div className="mt-4 p-3 bg-blue-950/30 border border-blue-600/50 rounded-lg">
        <p className="text-blue-400 text-sm mb-1">Next Turn</p>
        <p className="text-white">Turn right onto Highway 101 in 800m</p>
      </div>
    </Card>
  );
}
