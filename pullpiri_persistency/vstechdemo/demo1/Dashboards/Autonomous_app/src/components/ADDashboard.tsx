import { useState, useEffect } from 'react';
import { Card } from './ui/card';
import { Badge } from './ui/badge';
import { Alert, AlertDescription } from './ui/alert';
import { Activity, AlertTriangle } from 'lucide-react';
import { SpeedGauge } from './SpeedGauge';
import { LaneVisualizer } from './LaneVisualizer';
import { SteeringAngle } from './SteeringAngle';
import { ObstacleDetector } from './ObstacleDetector';
import { ParameterCard } from './ParameterCard';

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


export interface ADParameters {
  vehicle_speed: number;
  lane_position: number; // -1 to 1, 0 is center
  obstacle_detected: boolean;
  obstacle_distance: number;
  traffic_signal: 'green' | 'yellow' | 'red' | 'none';
  steering_angle: number; // degrees
  brake_force: number;
  acceleration: number;
  weather_condition: string;
  road_condition: string;
  timestamp: number;
  is_valid: boolean;
}

export function ADDashboard() {
  const [parameters, setParameters] = useState<ADParameters | null>(null);
  const [connectionStatus, setConnectionStatus] = useState<'connected' | 'disconnected' | 'error'>('disconnected');
  const [mountKey, setMountKey] = useState(Date.now());
  const backendUrl = import.meta.env?.VITE_BACKEND_URL || 'http://localhost:9080/data';

  // Fetch data from backend endpoint periodically
  useEffect(() => {
    // Set this dashboard as active
    DashboardRefreshManager.setActiveDashboard('autonomous');

    const fetchData = async () => {
      try {
        const response = await fetch(backendUrl);
        
        if (!response.ok) {
          throw new Error('Failed to fetch data');
        }
        
        const data: ADParameters = await response.json();
        console.log(data);
        setParameters(data);
        setConnectionStatus('connected');
      } catch (error) {
        console.error('Error fetching autonomous driving data:', error);
        setConnectionStatus('error');
      }
    };

    // Check if we should refresh based on dashboard switching
    const shouldRefresh = DashboardRefreshManager.shouldRefreshDashboard('autonomous');
    if (shouldRefresh) {
      console.log('Autonomous dashboard - forcing refresh due to dashboard switch');
      setParameters(null);
      setConnectionStatus('disconnected');
    }

    // Initial fetch
    fetchData();

    // Poll every 500ms for real-time updates
    const interval = setInterval(fetchData, 500);

    // Add visibility change listener to detect tab/window switching
    const handleVisibilityChange = () => {
      if (!document.hidden) {
        console.log('Autonomous dashboard became visible - refreshing data');
        DashboardRefreshManager.setActiveDashboard('autonomous');
        setParameters(null);
        setConnectionStatus('disconnected');
        fetchData();
      }
    };

    document.addEventListener('visibilitychange', handleVisibilityChange);

    return () => {
      clearInterval(interval);
      document.removeEventListener('visibilitychange', handleVisibilityChange);
    };
  }, [backendUrl]);

  // Show loading state if no data yet
  if (!parameters) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-slate-900 via-slate-800 to-slate-900 flex items-center justify-center">
        <div className="text-center">
          <Activity className="w-12 h-12 text-blue-400 animate-pulse mx-auto mb-4" />
          <p className="text-white">Connecting to Autonomous Driving System...</p>
          <p className="text-slate-400 text-sm mt-2">Waiting for data from {backendUrl}</p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Backend Connection Alert */}
      {connectionStatus !== 'connected' && (
        <Alert className="bg-red-950/50 border-red-600 text-red-400">
          <AlertTriangle className="h-5 w-5" />
          <AlertDescription className="ml-2">
            <span className="font-semibold">Not Connected to Backend</span> - Unable to reach backend server . 
            {connectionStatus === 'disconnected' ? ' Attempting to connect...' : ' Connection failed. Retrying...'}
            {' '}Displaying last received data.
          </AlertDescription>
        </Alert>
      )}

      {/* Status Bar */}
      <Card className="p-4 bg-green-950/30 border-green-600/50">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Activity className="w-5 h-5 text-green-500 animate-pulse" />
            <span className="text-green-400">Autonomous Driving Mode: ACTIVE</span>
            <Badge 
              variant={connectionStatus === 'connected' ? 'default' : 'destructive'}
              className={connectionStatus === 'connected' ? 'bg-blue-600' : ''}
            >
              {connectionStatus === 'connected' ? 'Backend Connected' : connectionStatus === 'error' ? 'Connection Error' : 'Connecting...'}
            </Badge>
          </div>
          <div className="flex items-center gap-3">
            <Badge variant={(parameters.is_valid ?? false) ? "default" : "destructive"} className={(parameters.is_valid ?? false) ? "bg-green-600" : ""}>
              {(parameters.is_valid ?? false) ? "Valid Data" : "Invalid Data"}
            </Badge>
            <span className="text-slate-400 text-sm">
              {parameters.timestamp ? new Date(parameters.timestamp).toLocaleTimeString() : 'No timestamp'}
            </span>
          </div>
        </div>
      </Card>

      {/* Main Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Speed Gauge */}
        <SpeedGauge currentSpeed={parameters.vehicle_speed ?? 0} />

        {/* Steering Angle */}
        <SteeringAngle steeringAngle={parameters.steering_angle ?? 0} />

        {/* Lane Position Visualizer */}
        <LaneVisualizer 
          lanePosition={parameters.lane_position ?? 0}
          steeringAngle={parameters.steering_angle ?? 0}
        />
      </div>

      {/* Obstacle Detection */}
      <ObstacleDetector 
        obstacleDetected={parameters.obstacle_detected ?? false}
        obstacleDistance={parameters.obstacle_distance ?? 0}
      />

      {/* Parameter Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <ParameterCard
          title="Traffic Signal"
          value={parameters.traffic_signal?.toUpperCase() || 'UNKNOWN'}
          status={parameters.traffic_signal || 'none'}
          unit=""
        />
        
        <ParameterCard
          title="Brake Force"
          value={parameters.brake_force?.toFixed(2) || '0.00'}
          unit=""
          status={(parameters.brake_force ?? 0) > 80 ? 'warning' : 'normal'}
        />
        
        <ParameterCard
          title="Acceleration"
          value={parameters.acceleration?.toFixed(1) || '0.0'}
          unit=""
          status={(parameters.acceleration ?? 0) < -5 ? 'warning' : 'normal'}
        />
      </div>

      {/* Environmental Conditions */}
      <Card className="p-6 bg-slate-800/50 border-slate-700">
        <h3 className="text-white mb-4">Environmental Conditions</h3>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div>
            <p className="text-slate-400 text-sm mb-1">Weather Condition</p>
            <p className="text-white text-xl capitalize">{parameters.weather_condition || 'Unknown'}</p>
          </div>
          <div>
            <p className="text-slate-400 text-sm mb-1">Road Condition</p>
            <p className="text-white text-xl capitalize">{parameters.road_condition || 'Unknown'}</p>
          </div>
        </div>
      </Card>
    </div>
  );
}