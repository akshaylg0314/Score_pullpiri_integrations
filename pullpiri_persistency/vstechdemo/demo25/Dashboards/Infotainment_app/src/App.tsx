import { useState, useEffect } from 'react';
import { NavigationMap } from './components/NavigationMap';
import { MusicPlayer } from './components/MusicPlayer';
import { HVACControl } from './components/HVACControl';
import { MoviesPlayer } from './components/MoviesPlayer';
import { EmergencyAlert } from './components/EmergencyAlert';
import { Car } from 'lucide-react';

export default function App() {
  const [showAlert, setShowAlert] = useState(false);
  const EMERGENCY_SERVER_URL = import.meta.env.VITE_EMERGENCY_SERVER_URL || 'http://192.168.2.177:9085/emergency';

  useEffect(() => {
    // Monitor emergency flag - show alert when systemctl service is triggered
    const checkEmergency = async () => {
      try {
        console.log('Checking emergency status...', EMERGENCY_SERVER_URL);
        const response = await fetch(EMERGENCY_SERVER_URL, {
          method: 'GET',
          headers: {
            'Content-Type': 'application/json',
          },
          mode: 'cors'
        });
        
        console.log('Response status:', response.status, response.statusText);
        
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        const data = await response.json();
        console.log('Emergency status response:', data);

        if (data && data.emergency_active && !showAlert) {
          console.log('Emergency detected! Showing alert...');
          setShowAlert(true);
          playAlertSound();
        } else if (data && !data.emergency_active && showAlert) {
          console.log('Emergency cleared, hiding alert...');
          setShowAlert(false);
        }
      } catch (error) {
        console.error('Failed to check emergency status:', error);
        console.error('Error details:', error.message);
        // If service is down or unreachable, clear the emergency popup
        if (showAlert) {
          console.log('Service unreachable, clearing emergency alert...');
          setShowAlert(false);
        }
      }
    };

    // Initial check
    checkEmergency();

  // Check every 0.5 seconds
  const interval = setInterval(checkEmergency, 500);

    return () => {
      clearInterval(interval);
    };
  }, [showAlert]);



  const playAlertSound = () => {
    // Create and play alert sound using Web Audio API
    const audioContext = new (window.AudioContext || (window as any).webkitAudioContext)();
    const oscillator = audioContext.createOscillator();
    const gainNode = audioContext.createGain();
    
    oscillator.connect(gainNode);
    gainNode.connect(audioContext.destination);
    
    oscillator.frequency.setValueAtTime(800, audioContext.currentTime);
    oscillator.type = 'sine';
    
    gainNode.gain.setValueAtTime(0.3, audioContext.currentTime);
    gainNode.gain.exponentialRampToValueAtTime(0.01, audioContext.currentTime + 0.5);
    
    oscillator.start(audioContext.currentTime);
    oscillator.stop(audioContext.currentTime + 0.5);
    
    // Second beep
    setTimeout(() => {
      const oscillator2 = audioContext.createOscillator();
      const gainNode2 = audioContext.createGain();
      
      oscillator2.connect(gainNode2);
      gainNode2.connect(audioContext.destination);
      
      oscillator2.frequency.setValueAtTime(800, audioContext.currentTime);
      oscillator2.type = 'sine';
      
      gainNode2.gain.setValueAtTime(0.3, audioContext.currentTime);
      gainNode2.gain.exponentialRampToValueAtTime(0.01, audioContext.currentTime + 0.5);
      
      oscillator2.start(audioContext.currentTime);
      oscillator2.stop(audioContext.currentTime + 0.5);
    }, 600);
  };

  const clearEmergency = async () => {
    try {
      const clearUrl = EMERGENCY_SERVER_URL.replace('/emergency', '/emergency/clear');
      await fetch(clearUrl, { method: 'POST' });
      setShowAlert(false);
    } catch (error) {
      console.error('Failed to clear emergency:', error);
    }
  };



  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-slate-800 to-slate-900 relative overflow-hidden">
      {/* Emergency Alert Overlay */}
      {showAlert && <EmergencyAlert mode="emergency" />}
      <div className="container mx-auto p-6">
        {/* Header */}
        <div className="flex items-center justify-between mb-8">
          <div className="flex items-center gap-3">
            <div className="p-3 bg-blue-600 rounded-lg">
              <Car className="w-8 h-8 text-white" />
            </div>
            <div>
              <h1 className="text-white">SDV Infotainment System</h1>
              <p className="text-slate-400">Vehicle Control & Entertainment</p>
            </div>
          </div>
        </div>
        {/* Main Dashboard Grid */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
          {/* Navigation Map */}
          <NavigationMap />
          {/* Music Player */}
          <MusicPlayer />
          {/* HVAC Control */}
          <HVACControl />
          {/* Movies Player */}
          <MoviesPlayer />
        </div>
      </div>
    </div>
  );
}