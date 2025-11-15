# Backend Integration Guide

## Payload Structure

The application expects data in the following exact format:

```typescript
interface ADParameters {
  vehicle_speed: number;
  lane_position: number;
  obstacle_detected: boolean;
  obstacle_distance: number;
  traffic_signal: 'green' | 'yellow' | 'red' | 'none';
  steering_angle: number;
  brake_force: number;
  acceleration: number;
  weather_condition: string;
  road_condition: string;
  timestamp: number;
  is_valid: boolean;
}
```

## Example Payload

```json
{
  "vehicle_speed": 71.0,
  "lane_position": -0.42,
  "obstacle_detected": true,
  "obstacle_distance": 9.177,
  "traffic_signal": "green",
  "steering_angle": 1.0,
  "brake_force": 104.115,
  "acceleration": -1.0,
  "weather_condition": "clear",
  "road_condition": "dry",
  "timestamp": 1762927220853,
  "is_valid": true
}
```

## Integration Steps

### Option 1: WebSocket Integration (Recommended for Real-Time)

Replace the simulation in `/components/ADDashboard.tsx`:

```typescript
useEffect(() => {
  const ws = new WebSocket('ws://your-backend-url/ad-data');
  
  ws.onmessage = (event) => {
    const data = JSON.parse(event.data);
    setParameters(data);
  };
  
  ws.onerror = (error) => {
    console.error('WebSocket error:', error);
  };
  
  return () => ws.close();
}, []);
```

### Option 2: HTTP Polling

```typescript
useEffect(() => {
  const fetchData = async () => {
    try {
      const response = await fetch('http://your-backend-url/api/ad-data');
      const data = await response.json();
      setParameters(data);
    } catch (error) {
      console.error('Failed to fetch AD data:', error);
    }
  };
  
  const interval = setInterval(fetchData, 500); // Poll every 500ms
  return () => clearInterval(interval);
}, []);
```

### Option 3: Server-Sent Events (SSE)

```typescript
useEffect(() => {
  const eventSource = new EventSource('http://your-backend-url/api/ad-stream');
  
  eventSource.onmessage = (event) => {
    const data = JSON.parse(event.data);
    setParameters(data);
  };
  
  eventSource.onerror = (error) => {
    console.error('SSE error:', error);
    eventSource.close();
  };
  
  return () => eventSource.close();
}, []);
```

## Field Specifications

| Field | Type | Range/Values | Description |
|-------|------|--------------|-------------|
| `vehicle_speed` | number | 0-120+ | Current speed in km/h |
| `lane_position` | number | -1.0 to 1.0 | Position in lane (0 = center) |
| `obstacle_detected` | boolean | true/false | Whether obstacle is detected |
| `obstacle_distance` | number | 0-50+ | Distance to obstacle in meters (0 if none) |
| `traffic_signal` | string | 'green', 'yellow', 'red', 'none' | Current traffic signal state |
| `steering_angle` | number | -45 to 45 | Steering angle in degrees |
| `brake_force` | number | 0-150+ | Brake force applied |
| `acceleration` | number | -10 to 10+ | Acceleration value |
| `weather_condition` | string | any | e.g., "clear", "rain", "fog" |
| `road_condition` | string | any | e.g., "dry", "wet", "icy" |
| `timestamp` | number | milliseconds | Unix timestamp in milliseconds |
| `is_valid` | boolean | true/false | Data validity flag |

## Visual Indicators

### Speed Gauge
- Displays `vehicle_speed` on circular gauge
- Max scale: 120 km/h

### Lane Visualizer
- Shows `lane_position` (-1 = left, 0 = center, 1 = right)
- Displays `steering_angle` with visual indicator
- Green when well-centered (|position| < 0.3)
- Orange when drifting

### Obstacle Detector
- Shows `obstacle_detected` status
- Displays `obstacle_distance` when detected
- Color-coded by distance:
  - **Red (Critical)**: < 15m
  - **Orange (Warning)**: 15-30m
  - **Yellow (Caution)**: 30-45m
  - **Green (Safe)**: > 45m or clear

### Status Bar
- Shows `is_valid` with badge (green = valid, red = invalid)
- Displays `timestamp` as formatted time
- Live indicator when AD mode is active

### Parameter Cards
- **Traffic Signal**: Color-coded (green/yellow/red/gray)
- **Brake Force**: Warning if > 80
- **Acceleration**: Warning if < -5

### Environmental Conditions
- Displays `weather_condition` and `road_condition`
- Capitalizes values for display

## Error Handling

Add error handling for invalid or missing data:

```typescript
const validatePayload = (data: any): ADParameters | null => {
  if (
    typeof data.vehicle_speed !== 'number' ||
    typeof data.lane_position !== 'number' ||
    typeof data.obstacle_detected !== 'boolean' ||
    typeof data.obstacle_distance !== 'number' ||
    typeof data.timestamp !== 'number'
  ) {
    console.error('Invalid payload structure');
    return null;
  }
  return data as ADParameters;
};

// Use in your integration:
const validated = validatePayload(receivedData);
if (validated) {
  setParameters(validated);
}
```

## Testing

Use this sample data generator for testing:

```typescript
const generateTestPayload = (): ADParameters => ({
  vehicle_speed: Math.random() * 80,
  lane_position: (Math.random() - 0.5) * 1.6,
  obstacle_detected: Math.random() > 0.7,
  obstacle_distance: Math.random() * 50,
  traffic_signal: ['green', 'yellow', 'red', 'none'][Math.floor(Math.random() * 4)],
  steering_angle: (Math.random() - 0.5) * 90,
  brake_force: Math.random() * 120,
  acceleration: (Math.random() - 0.5) * 10,
  weather_condition: 'clear',
  road_condition: 'dry',
  timestamp: Date.now(),
  is_valid: true,
});
```
