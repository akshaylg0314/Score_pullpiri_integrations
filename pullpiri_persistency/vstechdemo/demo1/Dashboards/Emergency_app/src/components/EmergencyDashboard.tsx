import { EmergencyModeData } from '../App';
import { Card } from './ui/card';
import { Badge } from './ui/badge';
import { 
  AlertTriangle, 
  Activity, 
  Gauge, 
  Shield, 
  Navigation,
  AlertCircle,
  CheckCircle,
  User,
  Car,
  Wind,
  Zap,
  LifeBuoy
} from 'lucide-react';
import { CircularGauge } from './CircularGauge';
import { SteeringWheel } from './SteeringWheel';
import { ObstacleRadar } from './ObstacleRadar';
import { useEffect } from 'react';

// Simple dashboard refresh manager
const DashboardRefreshManager = {
  setActiveDashboard: (dashboardName: string) => {
    localStorage.setItem('activeDashboard', dashboardName);
    localStorage.setItem('dashboardSwitchTime', Date.now().toString());
  },
  
  shouldRefreshDashboard: (dashboardName: string) => {
    const lastActive = localStorage.getItem('activeDashboard');
    const switchTime = localStorage.getItem('dashboardSwitchTime');
    
    if (!lastActive || !switchTime || lastActive !== dashboardName) {
      return true;
    }
    
    const timeSinceSwitch = Date.now() - parseInt(switchTime);
    return timeSinceSwitch > 5000;
  }
};

interface EmergencyDashboardProps {
  data: EmergencyModeData;
  isConnected: boolean;
}

export function EmergencyDashboard({ data, isConnected }: EmergencyDashboardProps) {
  // Register this dashboard as active
  useEffect(() => {
    DashboardRefreshManager.setActiveDashboard('emergency');
    
    const handleVisibilityChange = () => {
      if (!document.hidden) {
        console.log('Emergency dashboard became visible - refreshing data');
        DashboardRefreshManager.setActiveDashboard('emergency');
      }
    };

    document.addEventListener('visibilitychange', handleVisibilityChange);
    return () => document.removeEventListener('visibilitychange', handleVisibilityChange);
  }, []);

  // Determine severity colors
  const getCollisionRiskColor = (risk: number) => {
    if (risk >= 80) return 'red';
    if (risk >= 50) return 'orange';
    if (risk >= 30) return 'yellow';
    return 'green';
  };

  const getSpeedColor = (speed: number) => {
    if (speed > 40) return 'red';
    if (speed > 25) return 'yellow';
    return 'green';
  };

  const getDistanceColor = (distance: number) => {
    if (distance < 2) return 'red';
    if (distance < 5) return 'yellow';
    return 'green';
  };

  const riskColor = getCollisionRiskColor(data.collision_risk);
  const speedColor = getSpeedColor(data.vehicle_speed);
  const distanceColor = getDistanceColor(data.obstacle_distance);

  // Format emergency type for display
  const formatEmergencyType = (type: string) => {
    if (!type || typeof type !== 'string') {
      return 'Unknown Emergency';
    }
    return type.split('_').map(word => 
      word.charAt(0).toUpperCase() + word.slice(1)
    ).join(' ');
  };

  // Determine if road is slippery
  const isSlipperyRoad = (data.emergency_type || '') === 'slippery_road';

  return (
    <div className="container mx-auto p-6">
      {/* Header with Emergency Status */}
      <div className="mb-8">
        <div className="flex items-center justify-between mb-6">
          <div className="flex items-center gap-3">
            <div className="p-3 bg-red-600 rounded-lg">
              <AlertTriangle className="w-8 h-8 text-white" />
            </div>
            <div>
              <h1 className="text-white">Emergency Driving Mode</h1>
              <p className="text-slate-400">Active Emergency Control System</p>
            </div>
          </div>
          <Badge className="bg-red-600 animate-pulse">
            <Activity className="w-4 h-4 mr-2" />
            EMERGENCY ACTIVE
          </Badge>
        </div>

        {/* Emergency Type Banner */}
        <Card className={`p-4 ${
          data.collision_risk >= 80 ? 'bg-red-950/30 border-red-600/50' :
          data.collision_risk >= 50 ? 'bg-orange-950/30 border-orange-600/50' :
          'bg-yellow-950/30 border-yellow-600/50'
        }`}>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <AlertCircle className={`w-5 h-5 ${
                data.collision_risk >= 80 ? 'text-red-400' :
                data.collision_risk >= 50 ? 'text-orange-400' :
                'text-yellow-400'
              } animate-pulse`} />
              <span className={`${
                data.collision_risk >= 80 ? 'text-red-400' :
                data.collision_risk >= 50 ? 'text-orange-400' :
                'text-yellow-400'
              }`}>
                Emergency Type: {formatEmergencyType(data.emergency_type || 'unknown')}
              </span>
            </div>
            <div className="flex items-center gap-2">
              {data.emergency_lights && (
                <Badge className="bg-red-600 animate-pulse">Emergency Lights ON</Badge>
              )}
              <Badge variant="secondary">
                Signal: {data.traffic_signal?.toUpperCase() || 'UNKNOWN'}
              </Badge>
            </div>
          </div>
        </Card>
      </div>

      {/* Critical Vehicle Parameters */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 mb-6">
        {/* Speed Gauge */}
        <Card className="p-6 bg-slate-800/50 border-slate-700">
          <div className="flex items-center gap-3 mb-4">
            <Gauge className={`w-5 h-5 ${
              speedColor === 'red' ? 'text-red-400' :
              speedColor === 'yellow' ? 'text-yellow-400' :
              'text-green-400'
            }`} />
            <h3 className="text-white">Vehicle Speed</h3>
          </div>
          <CircularGauge 
            value={data.vehicle_speed} 
            max={100} 
            unit="km/h"
            color={speedColor}
            label="Vehicle Speed"
            description="Compliance with speed limits and safe driving conditions"
          />
        </Card>

        {/* Steering Angle */}
        <Card className="p-6 bg-slate-800/50 border-slate-700">
          <div className="flex items-center gap-3 mb-4">
            <Navigation className="w-5 h-5 text-blue-400" />
            <h3 className="text-white">Steering Angle</h3>
          </div>
          <SteeringWheel angle={data.steering_angle} />
        </Card>

        {/* Brake Force */}
        <Card className="p-6 bg-slate-800/50 border-slate-700">
          <div className="flex items-center gap-3 mb-4">
            <AlertTriangle className="w-5 h-5 text-red-400" />
            <h3 className="text-white">Brake Force</h3>
          </div>
          <CircularGauge 
            value={data.brake_force} 
            max={100} 
            unit="%"
            color={data.brake_force > 70 ? 'red' : 'blue'}
            label="Brake Force"
            description="Emergency braking system active"
          />
        </Card>
      </div>

      {/* Safety Systems Status */}
      <div className="mb-6">
        <h2 className="text-white mb-4">Safety Systems</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          {/* Seatbelt Status */}
          <Card className={`p-4 ${
            data.seatbelt_tightened 
              ? 'bg-green-950/30 border-green-600/50' 
              : 'bg-slate-800/50 border-slate-700'
          }`}>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <Shield className={`w-5 h-5 ${
                  data.seatbelt_tightened ? 'text-green-400' : 'text-slate-400'
                }`} />
                <div>
                  <p className="text-slate-400 text-sm">Seatbelt</p>
                  <p className={`${
                    data.seatbelt_tightened ? 'text-green-400' : 'text-white'
                  }`}>
                    {data.seatbelt_tightened ? 'TIGHTENED' : 'Normal'}
                  </p>
                </div>
              </div>
              {data.seatbelt_tightened && (
                <CheckCircle className="w-5 h-5 text-green-400 animate-pulse" />
              )}
            </div>
          </Card>

          {/* Headrest Support */}
          <Card className="p-4 bg-green-950/30 border-green-600/50">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <LifeBuoy className="w-5 h-5 text-green-400" />
                <div>
                  <p className="text-slate-400 text-sm">Headrest Support</p>
                  <p className="text-green-400">ENABLED</p>
                </div>
              </div>
              <CheckCircle className="w-5 h-5 text-green-400 animate-pulse" />
            </div>
          </Card>

          {/* Stability Control */}
          <Card className={`p-4 ${
            data.stability_control 
              ? 'bg-green-950/30 border-green-600/50' 
              : 'bg-orange-950/30 border-orange-600/50'
          }`}>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <Activity className={`w-5 h-5 ${
                  data.stability_control ? 'text-green-400' : 'text-orange-400'
                }`} />
                <div>
                  <p className="text-slate-400 text-sm">Stability Control</p>
                  <p className={`${
                    data.stability_control ? 'text-green-400' : 'text-orange-400'
                  }`}>
                    {data.stability_control ? 'ACTIVE' : 'INACTIVE'}
                  </p>
                </div>
              </div>
              {data.stability_control && (
                <CheckCircle className="w-5 h-5 text-green-400 animate-pulse" />
              )}
            </div>
          </Card>

          {/* Airbag Status */}
          <Card className={`p-4 ${
            data.airbag_ready 
              ? 'bg-green-950/30 border-green-600/50' 
              : 'bg-red-950/30 border-red-600/50'
          }`}>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <Shield className={`w-5 h-5 ${
                  data.airbag_ready ? 'text-green-400' : 'text-red-400'
                }`} />
                <div>
                  <p className="text-slate-400 text-sm">Airbag System</p>
                  <p className={`${
                    data.airbag_ready ? 'text-green-400' : 'text-red-400'
                  }`}>
                    {data.airbag_ready ? 'READY' : 'NOT READY'}
                  </p>
                </div>
              </div>
              {data.airbag_ready && (
                <CheckCircle className="w-5 h-5 text-green-400" />
              )}
            </div>
          </Card>
        </div>
      </div>

      {/* Obstacle Detection & Collision Risk */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
        {/* Obstacle Radar */}
        <Card className="p-6 bg-slate-800/50 border-slate-700">
          <div className="flex items-center gap-3 mb-4">
            <Car className="w-5 h-5 text-blue-400" />
            <h3 className="text-white">Obstacle Detection</h3>
            {data.obstacle_detected && (
              <Badge className="bg-red-600 animate-pulse ml-auto">OBSTACLE DETECTED</Badge>
            )}
          </div>
          <ObstacleRadar 
            obstacleDetected={data.obstacle_detected}
            obstacleDistance={data.obstacle_distance}
          />
          <div className="mt-4 pt-4 border-t border-slate-700">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <p className="text-slate-400 text-sm mb-1">Status</p>
                <p className={`${
                  data.obstacle_detected ? 'text-red-400' : 'text-green-400'
                }`}>
                  {data.obstacle_detected ? 'DETECTED' : 'Clear'}
                </p>
              </div>
              <div>
                <p className="text-slate-400 text-sm mb-1">Distance</p>
                <p className={`${
                  distanceColor === 'red' ? 'text-red-400' :
                  distanceColor === 'yellow' ? 'text-yellow-400' :
                  'text-green-400'
                }`}>
                  {(data.obstacle_distance ?? 0).toFixed(1)} m
                </p>
              </div>
            </div>
          </div>
        </Card>

        {/* Collision Risk */}
        <Card className="p-6 bg-slate-800/50 border-slate-700">
          <div className="flex items-center gap-3 mb-4">
            <AlertTriangle className={`w-5 h-5 ${
              riskColor === 'red' ? 'text-red-400' :
              riskColor === 'orange' ? 'text-orange-400' :
              riskColor === 'yellow' ? 'text-yellow-400' :
              'text-green-400'
            }`} />
            <h3 className="text-white">Collision Risk Assessment</h3>
          </div>
          <CircularGauge 
            value={data.collision_risk} 
            max={100} 
            unit="%"
            color={riskColor}
          />
          <div className="mt-4 pt-4 border-t border-slate-700">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <p className="text-slate-400 text-sm mb-1">Risk Level</p>
                <p className={`${
                  riskColor === 'red' ? 'text-red-400' :
                  riskColor === 'orange' ? 'text-orange-400' :
                  riskColor === 'yellow' ? 'text-yellow-400' :
                  'text-green-400'
                }`}>
                  {data.collision_risk >= 80 ? 'CRITICAL' :
                   data.collision_risk >= 50 ? 'HIGH' :
                   data.collision_risk >= 30 ? 'MODERATE' : 'LOW'}
                </p>
              </div>
              <div>
                <p className="text-slate-400 text-sm mb-1">Emergency Brake</p>
                <p className="text-red-400">{(data.emergency_brake_force ?? 0).toFixed(0)}%</p>
              </div>
            </div>
          </div>
        </Card>
      </div>

      {/* Environmental Conditions */}
      <Card className="p-6 bg-slate-800/50 border-slate-700">
        <h3 className="text-white mb-4">Environmental Conditions</h3>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {/* Road Condition */}
          <div>
            <div className="flex items-center gap-2 mb-2">
              <Wind className={`w-5 h-5 ${
                isSlipperyRoad ? 'text-yellow-400' : 'text-green-400'
              }`} />
              <p className="text-slate-400 text-sm">Road Condition</p>
            </div>
            <div className={`p-3 rounded-lg ${
              isSlipperyRoad 
                ? 'bg-yellow-950/30 border border-yellow-600/50' 
                : 'bg-green-950/30 border border-green-600/50'
            }`}>
              <p className={`${
                isSlipperyRoad ? 'text-yellow-400' : 'text-green-400'
              }`}>
                {isSlipperyRoad ? '⚠️ SLIPPERY' : '✓ Normal'}
              </p>
            </div>
          </div>

          {/* Traffic Signal */}
          <div>
            <div className="flex items-center gap-2 mb-2">
              <Zap className="w-5 h-5 text-blue-400" />
              <p className="text-slate-400 text-sm">Traffic Signal</p>
            </div>
            <div className={`p-3 rounded-lg ${
              data.traffic_signal === 'emergency' 
                ? 'bg-red-950/30 border border-red-600/50' 
                : 'bg-orange-950/30 border border-orange-600/50'
            }`}>
              <p className={`${
                data.traffic_signal === 'emergency' ? 'text-red-400' : 'text-orange-400'
              }`}>
                {data.traffic_signal?.toUpperCase() || 'UNKNOWN'}
              </p>
            </div>
          </div>

          {/* System Validity */}
          <div>
            <div className="flex items-center gap-2 mb-2">
              <CheckCircle className={`w-5 h-5 ${
                data.is_valid ? 'text-green-400' : 'text-red-400'
              }`} />
              <p className="text-slate-400 text-sm">System Status</p>
            </div>
            <div className={`p-3 rounded-lg ${
              data.is_valid 
                ? 'bg-green-950/30 border border-green-600/50' 
                : 'bg-red-950/30 border border-red-600/50'
            }`}>
              <p className={`${
                data.is_valid ? 'text-green-400' : 'text-red-400'
              }`}>
                {data.is_valid ? '✓ Valid' : '✗ Invalid'}
              </p>
            </div>
          </div>
        </div>
      </Card>

      {/* Timestamp */}
      <div className="mt-4 text-center">
        <p className="text-slate-500 text-sm">
          Last Update: {data.timestamp ? new Date(data.timestamp).toLocaleTimeString() : 'No timestamp'}
        </p>
      </div>
    </div>
  );
}