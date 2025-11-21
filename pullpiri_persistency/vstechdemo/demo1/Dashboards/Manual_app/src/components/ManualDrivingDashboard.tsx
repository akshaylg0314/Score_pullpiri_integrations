import { useState, useEffect } from 'react';
import { Car, Activity, StopCircle, AlertTriangle, Gauge, Navigation, WifiOff } from 'lucide-react';
import { Button } from './ui/button';
import { Card } from './ui/card';
import { Badge } from './ui/badge';
import CircularGauge from './CircularGauge';
import SteeringIndicator from './SteeringIndicator';
import ParameterCard from './ParameterCard';

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

interface ManualCarData {
  vehicle_speed: number;
  steering_angle: number;
  brake_force: number;
  acceleration: number;
  weather_condition: string;
  road_condition: string;
  driver_alertness: boolean;
  throttle_position: number;
  timestamp: number;
  is_valid: boolean;
}

export default function ManualDrivingDashboard() {
  const [carData, setCarData] = useState<ManualCarData | null>(null);
  const [isBackendConnected, setIsBackendConnected] = useState(true);
  const [lastSuccessfulUpdate, setLastSuccessfulUpdate] = useState<number>(Date.now());
  const [mountKey, setMountKey] = useState(Date.now());

  const backendUrl = import.meta.env?.VITE_BACKEND_URL || 'http://localhost:9081/data';

  useEffect(() => {
    // Set this dashboard as active
    DashboardRefreshManager.setActiveDashboard('manual');

    const fetchData = async () => {
      try {
        const response = await fetch(backendUrl);
        if (!response.ok) {
          throw new Error('Backend response not OK');
        }
        const data: ManualCarData = await response.json();
        setCarData(data);
        setIsBackendConnected(true);
        setLastSuccessfulUpdate(Date.now());
      } catch (error) {
        console.error('Failed to fetch data from backend:', error);
        setIsBackendConnected(false);
      }
    };

    // Check if we should refresh based on dashboard switching
    const shouldRefresh = DashboardRefreshManager.shouldRefreshDashboard('manual');
    if (shouldRefresh) {
      console.log('Manual dashboard - forcing refresh due to dashboard switch');
      setCarData(null);
      setIsBackendConnected(false);
    }

    // Initial fetch
    fetchData();

    // Poll every 1 second
    const interval = setInterval(fetchData, 1000);

    // Add visibility change listener to detect tab/window switching
    const handleVisibilityChange = () => {
      if (!document.hidden) {
        console.log('Manual dashboard became visible - refreshing data');
        DashboardRefreshManager.setActiveDashboard('manual');
        setCarData(null);
        setIsBackendConnected(false);
        fetchData();
      }
    };

    document.addEventListener('visibilitychange', handleVisibilityChange);

    return () => {
      clearInterval(interval);
      document.removeEventListener('visibilitychange', handleVisibilityChange);
    };
  }, [backendUrl]);

  // If no data yet, show loading
  if (!carData) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-slate-900 via-slate-800 to-slate-900 flex items-center justify-center">
        <div className="text-center">
          <Activity className="w-12 h-12 text-blue-400 animate-pulse mx-auto mb-4" />
          <p className="text-white">Connecting to backend...</p>
          <p className="text-slate-400 text-sm mt-2">Waiting for data from {backendUrl}</p>
        </div>
      </div>
    );
  }

  const getSpeedStatus = (speed: number) => {
    if (speed > 100) return { color: 'red', label: 'Excessive' };
    if (speed > 80) return { color: 'orange', label: 'High' };
    if (speed > 40) return { color: 'green', label: 'Normal' };
    return { color: 'blue', label: 'Low' };
  };

  // Show loading state if no data yet
  if (!carData) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-slate-900 via-slate-800 to-slate-900 flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-white mx-auto mb-4"></div>
          <p className="text-white">Loading Manual Driving Dashboard...</p>
        </div>
      </div>
    );
  }

  const speedStatus = getSpeedStatus(carData.vehicle_speed);

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-slate-800 to-slate-900">
      <div className="container mx-auto p-6">
        {/* Header */}
        <div className="flex items-center justify-between mb-8">
          <div className="flex items-center gap-3">
            <div className="p-3 bg-green-600 rounded-lg">
              <Car className="w-8 h-8 text-white" />
            </div>
            <div>
              <h1 className="text-white">Manual Driving Mode</h1>
              <p className="text-slate-400">Real-time Vehicle Monitoring & Control</p>
            </div>
          </div>
        </div>

        {/* Backend Connection Warning */}
        {!isBackendConnected && (
          <Card className="p-4 bg-orange-950/30 border-orange-600/50 mb-6">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <WifiOff className="w-5 h-5 text-orange-500 animate-pulse" />
                <span className="text-orange-400">Not Connected to Backend - Displaying Last Known Data</span>
              </div>
              <Badge className="bg-orange-600">
                Offline
              </Badge>
            </div>
          </Card>
        )}

        {/* Active Status Banner */}
        <Card className="p-4 bg-green-950/30 border-green-600/50 mb-6">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <Activity className="w-5 h-5 text-green-500 animate-pulse" />
              <span className="text-green-400">Manual Driving Mode Active</span>
            </div>
            <div className="flex items-center gap-4">
              <div className="flex items-center gap-2">
                <div className="w-3 h-3 rounded-full bg-green-500 animate-pulse"></div>
                <span className="text-slate-400 text-sm">Live Data Stream</span>
              </div>
              <Badge className="bg-green-600">
                {carData.is_valid ? 'Valid Data' : 'Invalid Data'}
              </Badge>
            </div>
          </div>
        </Card>

        {/* Driver Alertness Warning */}
        {!carData.driver_alertness && (
          <Card className="p-4 bg-red-950/30 border-red-600/50 mb-6">
            <div className="flex items-center gap-3">
              <AlertTriangle className="w-5 h-5 text-red-500 animate-pulse" />
              <span className="text-red-400">Warning: Driver Alertness Low - Please Take Caution</span>
            </div>
          </Card>
        )}

        {/* Main Dashboard Grid */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
          {/* Vehicle Speed Gauge */}
          <Card className="p-6 bg-slate-800/50 border-slate-700">
            <div className="flex items-center gap-2 mb-4">
              <Gauge className="w-5 h-5 text-blue-400" />
              <h3 className="text-white">Vehicle Speed</h3>
            </div>
            <div className="flex flex-col items-center">
              <CircularGauge 
                value={carData.vehicle_speed} 
                max={120} 
                unit="km/h"
                color={speedStatus.color}
              />
              <Badge 
                className={`mt-4 ${
                  speedStatus.color === 'green' ? 'bg-green-600' : 
                  speedStatus.color === 'orange' ? 'bg-orange-600' : 
                  speedStatus.color === 'red' ? 'bg-red-600' : 'bg-blue-600'
                }`}
              >
                {speedStatus.label}
              </Badge>
              <p className="text-slate-400 text-sm mt-2">
                Compliance with speed limits and safe driving conditions
              </p>
            </div>
          </Card>

          {/* Steering Angle */}
          <Card className="p-6 bg-slate-800/50 border-slate-700">
            <div className="flex items-center gap-2 mb-4">
              <Navigation className="w-5 h-5 text-blue-400" />
              <h3 className="text-white">Steering Angle</h3>
            </div>
            <div className="flex flex-col items-center">
              <SteeringIndicator angle={carData.steering_angle} />
              <div className="mt-4">
                <p className="text-slate-400 text-sm mb-1 text-center">Current Angle</p>
                <p className="text-white text-center">{(carData.steering_angle ?? 0).toFixed(1)}°</p>
              </div>
              <p className="text-slate-400 text-sm mt-4 text-center">
                Adjusts steering to navigate turns and curves safely
              </p>
            </div>
          </Card>
        </div>

        {/* Braking and Acceleration */}
        <Card className="p-6 bg-slate-800/50 border-slate-700 mb-6">
          <h3 className="text-white mb-4">Braking and Acceleration Control</h3>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            <ParameterCard
              label="Brake Force"
              value={(carData.brake_force ?? 0).toFixed(1)}
              unit="%"
              icon="brake"
            />
            <ParameterCard
              label="Acceleration"
              value={(carData.acceleration ?? 0).toFixed(2)}
              unit="m/s²"
              icon="acceleration"
            />
            <ParameterCard
              label="Throttle Position"
              value={(carData.throttle_position ?? 0).toFixed(1)}
              unit="%"
              icon="throttle"
            />
            <ParameterCard
              label="Driver Alertness"
              value={carData.driver_alertness ? 'Alert' : 'Warning'}
              unit=""
              icon="alertness"
              status={carData.driver_alertness ? 'good' : 'critical'}
            />
          </div>
          <p className="text-slate-400 text-sm mt-4">
            Controls braking and acceleration for smooth and safe driving
          </p>
        </Card>

        {/* Environmental Conditions */}
        <Card className="p-6 bg-slate-800/50 border-slate-700">
          <h3 className="text-white mb-4">Environmental Conditions</h3>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-4">
            <div>
              <p className="text-slate-400 text-sm mb-1">Weather Condition</p>
              <p className="text-white capitalize">{carData.weather_condition}</p>
            </div>
            <div>
              <p className="text-slate-400 text-sm mb-1">Road Condition</p>
              <p className="text-white capitalize">{carData.road_condition}</p>
            </div>
            <div>
              <p className="text-slate-400 text-sm mb-1">Last Update</p>
              <p className="text-white">{carData.timestamp ? new Date(carData.timestamp).toLocaleTimeString() : 'No timestamp'}</p>
            </div>
          </div>
          <p className="text-slate-400 text-sm">
            Monitors weather and road conditions to adjust driving behavior accordingly
          </p>
        </Card>
      </div>
    </div>
  );
}