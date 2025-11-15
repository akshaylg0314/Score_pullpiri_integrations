import { useState, useEffect } from 'react';
import { EmergencyDashboard } from './components/EmergencyDashboard';
import { Alert, AlertDescription } from './components/ui/alert';
import { AlertCircle, WifiOff } from 'lucide-react';

// Emergency Mode Data interface based on backend payload
export interface EmergencyModeData {
  vehicle_speed: number;
  steering_angle: number;
  brake_force: number;
  obstacle_detected: boolean;
  obstacle_distance: number;
  collision_risk: number;
  stability_control: boolean;
  traffic_signal: string;
  seatbelt_tightened: boolean;
  emergency_lights: boolean;
  emergency_type: string;
  emergency_brake_force: number;
  airbag_ready: boolean;
  timestamp: number;
  is_valid: boolean;
}

// Get backend URL with fallback
const BACKEND_URL = import.meta.env?.VITE_BACKEND_URL || 'http://localhost:9082/data';

function App() {
  const [emergencyData, setEmergencyData] = useState<EmergencyModeData | null>(null);
  const [isConnected, setIsConnected] = useState(true);
  const [lastSuccessfulFetch, setLastSuccessfulFetch] = useState<number>(Date.now());

  // Fetch data from backend
  const fetchEmergencyData = async () => {
    try {
      const response = await fetch(BACKEND_URL, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
        },
        signal: AbortSignal.timeout(5000), // 5 second timeout
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data: EmergencyModeData = await response.json();
      setEmergencyData(data);
      setIsConnected(true);
      setLastSuccessfulFetch(Date.now());
    } catch (error) {
      console.error('Failed to fetch emergency data:', error);
      setIsConnected(false);
      
      // If this is the first fetch and it fails, set mock data
      if (!emergencyData) {
        setEmergencyData({
          vehicle_speed: 0,
          steering_angle: 0,
          brake_force: 0,
          obstacle_detected: false,
          obstacle_distance: 100,
          collision_risk: 0,
          stability_control: true,
          traffic_signal: "normal",
          seatbelt_tightened: false,
          emergency_lights: false,
          emergency_type: "none",
          emergency_brake_force: 0,
          airbag_ready: true,
          timestamp: Date.now(),
          is_valid: false
        });
      }
    }
  };

  // Fetch data on mount and periodically
  useEffect(() => {
    // Reset data to null on startup to refresh dashboard
    setEmergencyData(null);
    setIsConnected(false);

    // Initial fetch
    fetchEmergencyData();

    // Set up polling interval (every 1 second)
    const interval = setInterval(() => {
      fetchEmergencyData();
    }, 1000);

    return () => clearInterval(interval);
  }, []);

  // Calculate time since last successful fetch
  const getTimeSinceLastUpdate = () => {
    const seconds = Math.floor((Date.now() - lastSuccessfulFetch) / 1000);
    if (seconds < 60) return `${seconds}s ago`;
    const minutes = Math.floor(seconds / 60);
    return `${minutes}m ago`;
  };

  if (!emergencyData) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-slate-900 via-slate-800 to-slate-900 flex items-center justify-center">
        <div className="text-center">
          <div className="w-16 h-16 border-4 border-blue-600 border-t-transparent rounded-full animate-spin mx-auto mb-4"></div>
          <p className="text-white">Connecting to Emergency System...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-slate-800 to-slate-900">
      {/* Connection Status Alert */}
      {!isConnected && (
        <div className="sticky top-0 z-50 p-4">
          <Alert className="bg-red-950/90 border-red-600/50 backdrop-blur-sm">
            <WifiOff className="h-5 w-5 text-red-400" />
            <AlertDescription className="ml-2 text-red-400">
              <div className="flex items-center justify-between">
                <div>
                  <span className="font-semibold">Backend Connection Lost</span> - Displaying last known data from {getTimeSinceLastUpdate()}
                </div>
                <div className="flex items-center gap-2">
                  <div className="w-2 h-2 rounded-full bg-red-500 animate-pulse"></div>
                  <span className="text-sm">Reconnecting...</span>
                </div>
              </div>
            </AlertDescription>
          </Alert>
        </div>
      )}
      
      <EmergencyDashboard data={emergencyData} isConnected={isConnected} />
    </div>
  );
}

export default App;