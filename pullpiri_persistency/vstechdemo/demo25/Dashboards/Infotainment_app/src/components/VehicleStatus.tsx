import { Card } from './ui/card';
import { Car, Battery, Gauge, Zap, Activity } from 'lucide-react';

interface VehicleData {
  vehicle_speed: number;
  steering_angle?: number;
  brake_force?: number;
  acceleration?: number;
  battery_level?: number;
  weather_condition?: string;
  road_condition?: string;
  driver_alertness?: boolean;
  throttle_position?: number;
  lane_position?: number;
  obstacle_detected?: boolean;
  obstacle_distance?: number;
  traffic_signal?: string;
  timestamp: number;
  is_valid: boolean;
}

interface VehicleStatusProps {
  drivingMode: 'autonomous' | 'manual' | 'emergency';
  vehicleData?: VehicleData | null;
}

export function VehicleStatus({ drivingMode, vehicleData }: VehicleStatusProps) {
  const batteryLevel = vehicleData?.battery_level || 78;
  const speed = vehicleData?.vehicle_speed || (drivingMode === 'emergency' ? 35 : drivingMode === 'manual' ? 55 : 65);
  const range = 245;

  return (
    <Card className="p-6 bg-slate-800/50 border-slate-700">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <Car className="w-5 h-5 text-blue-400" />
          <h3 className="text-white">Vehicle Status</h3>
        </div>
        <div className="flex items-center gap-2">
          <Activity className={`w-5 h-5 ${
            drivingMode === 'autonomous' ? 'text-green-500' :
            drivingMode === 'manual' ? 'text-yellow-500' :
            'text-red-500'
          } ${drivingMode !== 'autonomous' ? 'animate-pulse' : ''}`} />
          <span className={`text-sm ${
            drivingMode === 'autonomous' ? 'text-green-400' :
            drivingMode === 'manual' ? 'text-yellow-400' :
            'text-red-400'
          }`}>
            {drivingMode.charAt(0).toUpperCase() + drivingMode.slice(1)}
          </span>
        </div>
      </div>

      {/* Speed Gauge */}
      <div className="mb-6">
        <div className="relative">
          <div className="flex items-center justify-center">
            <div className="relative w-48 h-48">
              <svg className="w-full h-full" viewBox="0 0 200 200">
                {/* Background Circle */}
                <circle
                  cx="100"
                  cy="100"
                  r="80"
                  fill="none"
                  stroke="#334155"
                  strokeWidth="16"
                />
                {/* Progress Circle */}
                <circle
                  cx="100"
                  cy="100"
                  r="80"
                  fill="none"
                  stroke={
                    drivingMode === 'emergency' ? '#ef4444' :
                    drivingMode === 'manual' ? '#eab308' :
                    '#3b82f6'
                  }
                  strokeWidth="16"
                  strokeDasharray={`${(speed / 140) * 502} 502`}
                  strokeLinecap="round"
                  transform="rotate(-90 100 100)"
                  className="transition-all duration-500"
                />
              </svg>
              <div className="absolute inset-0 flex flex-col items-center justify-center">
                <span className="text-white text-5xl">{speed.toFixed(0)}</span>
                <span className="text-slate-400">km/h</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Status Grid */}
      <div className="grid grid-cols-2 gap-4 mb-4">
        {/* Battery */}
        <div className="bg-slate-900/50 p-4 rounded-lg">
          <div className="flex items-center gap-2 mb-2">
            <Battery className="w-5 h-5 text-green-400" />
            <p className="text-slate-400 text-sm">Battery</p>
          </div>
          <p className="text-white text-2xl mb-1">{batteryLevel}%</p>
          <div className="h-2 bg-slate-700 rounded-full overflow-hidden">
            <div 
              className="h-full bg-green-500 transition-all duration-300"
              style={{ width: `${batteryLevel}%` }}
            />
          </div>
        </div>

        {/* Range */}
        <div className="bg-slate-900/50 p-4 rounded-lg">
          <div className="flex items-center gap-2 mb-2">
            <Gauge className="w-5 h-5 text-blue-400" />
            <p className="text-slate-400 text-sm">Range</p>
          </div>
          <p className="text-white text-2xl">{range} km</p>
          <p className="text-slate-500 text-sm">Est. remaining</p>
        </div>
      </div>

      {/* Additional Metrics */}
      <div className="grid grid-cols-3 gap-3">
        <div className="bg-slate-900/50 p-3 rounded-lg text-center">
          <Zap className="w-4 h-4 text-yellow-400 mx-auto mb-1" />
          <p className="text-slate-400 text-xs mb-1">Steering</p>
          <p className="text-white text-sm">{vehicleData?.steering_angle?.toFixed(1) || '--'}°</p>
        </div>
        <div className="bg-slate-900/50 p-3 rounded-lg text-center">
          <Activity className="w-4 h-4 text-blue-400 mx-auto mb-1" />
          <p className="text-slate-400 text-xs mb-1">Brake</p>
          <p className="text-white text-sm">{vehicleData?.brake_force?.toFixed(0) || '--'}%</p>
        </div>
        <div className="bg-slate-900/50 p-3 rounded-lg text-center">
          <Gauge className="w-4 h-4 text-green-400 mx-auto mb-1" />
          <p className="text-slate-400 text-xs mb-1">Accel</p>
          <p className="text-white text-sm">{vehicleData?.acceleration?.toFixed(1) || '--'}m/s²</p>
        </div>
      </div>

      {/* System Status */}
      <div className="mt-4 pt-4 border-t border-slate-700">
        <p className="text-slate-400 text-sm mb-2">System Status</p>
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <span className="text-slate-400 text-sm">Autonomous Systems</span>
            <div className="flex items-center gap-2">
              <div className={`w-2 h-2 rounded-full ${
                drivingMode === 'autonomous' ? 'bg-green-500 animate-pulse' : 'bg-slate-600'
              }`} />
              <span className={`text-sm ${
                drivingMode === 'autonomous' ? 'text-green-400' : 'text-slate-500'
              }`}>
                {drivingMode === 'autonomous' ? 'Active' : 'Inactive'}
              </span>
            </div>
          </div>
          <div className="flex items-center justify-between">
            <span className="text-slate-400 text-sm">Safety Systems</span>
            <div className="flex items-center gap-2">
              <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse" />
              <span className="text-sm text-green-400">Active</span>
            </div>
          </div>
          <div className="flex items-center justify-between">
            <span className="text-slate-400 text-sm">Connectivity</span>
            <div className="flex items-center gap-2">
              <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse" />
              <span className="text-sm text-green-400">5G Connected</span>
            </div>
          </div>
        </div>
      </div>
    </Card>
  );
}
