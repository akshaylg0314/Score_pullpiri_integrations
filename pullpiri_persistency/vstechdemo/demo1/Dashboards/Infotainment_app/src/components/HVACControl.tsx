import { useState } from 'react';
import { Card } from './ui/card';
import { Wind, Thermometer, Droplets, Fan, Power } from 'lucide-react';
import { Button } from './ui/button';
import { Slider } from './ui/slider';

export function HVACControl() {
  const [isOn, setIsOn] = useState(true);
  const [temperature, setTemperature] = useState([22]);
  const [fanSpeed, setFanSpeed] = useState([3]);
  const [mode, setMode] = useState<'auto' | 'cool' | 'heat'>('auto');
  const [humidity, setHumidity] = useState(45);

  const modes = [
    { id: 'auto', label: 'Auto', icon: Wind },
    { id: 'cool', label: 'Cool', icon: Thermometer },
    { id: 'heat', label: 'Heat', icon: Thermometer },
  ];

  return (
    <Card className="p-6 bg-slate-800/50 border-slate-700">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <Wind className="w-5 h-5 text-blue-400" />
          <h3 className="text-white">Climate Control</h3>
        </div>
        <Button
          variant={isOn ? "default" : "ghost"}
          size="sm"
          onClick={() => setIsOn(!isOn)}
          className={isOn ? 'bg-green-600 hover:bg-green-700' : ''}
        >
          <Power className="w-4 h-4 mr-2" />
          {isOn ? 'ON' : 'OFF'}
        </Button>
      </div>

      {/* Temperature Display */}
      <div className="mb-6">
        <div className="flex items-center justify-center mb-4">
          <div className="text-center">
            <p className="text-slate-400 text-sm mb-2">Target Temperature</p>
            <div className="flex items-baseline justify-center gap-1">
              <span className="text-white text-6xl">{temperature[0]}</span>
              <span className="text-slate-400 text-3xl">°C</span>
            </div>
          </div>
        </div>

        {/* Temperature Slider */}
        <div className="mb-2">
          <Slider
            value={temperature}
            onValueChange={setTemperature}
            min={16}
            max={30}
            step={0.5}
            disabled={!isOn}
            className="w-full"
          />
        </div>
        <div className="flex justify-between text-slate-500 text-sm">
          <span>16°C</span>
          <span>30°C</span>
        </div>
      </div>

      {/* Mode Selection */}
      <div className="mb-6">
        <p className="text-slate-400 text-sm mb-2">Climate Mode</p>
        <div className="grid grid-cols-3 gap-2">
          {modes.map((m) => {
            const Icon = m.icon;
            return (
              <Button
                key={m.id}
                variant="ghost"
                onClick={() => setMode(m.id as any)}
                disabled={!isOn}
                className={`${
                  mode === m.id && isOn
                    ? 'bg-blue-950/50 border-blue-600/50 text-blue-400'
                    : 'bg-slate-900/50 text-slate-400'
                } border transition-all`}
              >
                <Icon className="w-4 h-4 mr-2" />
                {m.label}
              </Button>
            );
          })}
        </div>
      </div>

      {/* Fan Speed */}
      <div className="mb-6">
        <div className="flex items-center justify-between mb-2">
          <div className="flex items-center gap-2">
            <Fan className={`w-5 h-5 text-blue-400 ${isOn ? 'animate-spin' : ''}`} 
                 style={{ animationDuration: `${4 - fanSpeed[0]}s` }} />
            <p className="text-slate-400 text-sm">Fan Speed</p>
          </div>
          <span className="text-white">{fanSpeed[0]}/5</span>
        </div>
        <Slider
          value={fanSpeed}
          onValueChange={setFanSpeed}
          min={1}
          max={5}
          step={1}
          disabled={!isOn}
          className="w-full"
        />
      </div>

      {/* Additional Info */}
      <div className="grid grid-cols-2 gap-4">
        <div className="bg-slate-900/50 p-3 rounded-lg">
          <div className="flex items-center gap-2 mb-1">
            <Thermometer className="w-4 h-4 text-blue-400" />
            <p className="text-slate-400 text-sm">Cabin Temp</p>
          </div>
          <p className="text-white">21.5°C</p>
        </div>
        <div className="bg-slate-900/50 p-3 rounded-lg">
          <div className="flex items-center gap-2 mb-1">
            <Droplets className="w-4 h-4 text-blue-400" />
            <p className="text-slate-400 text-sm">Humidity</p>
          </div>
          <p className="text-white">{humidity}%</p>
        </div>
      </div>

      {/* Zone Controls */}
      <div className="mt-4 pt-4 border-t border-slate-700">
        <p className="text-slate-400 text-sm mb-3">Zone Control</p>
        <div className="grid grid-cols-2 gap-2">
          <Button variant="ghost" size="sm" className="bg-slate-900/50 text-slate-400 hover:text-white">
            Driver: 22°C
          </Button>
          <Button variant="ghost" size="sm" className="bg-slate-900/50 text-slate-400 hover:text-white">
            Passenger: 23°C
          </Button>
        </div>
      </div>
    </Card>
  );
}
