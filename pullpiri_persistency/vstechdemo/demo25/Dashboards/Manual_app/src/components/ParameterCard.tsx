import { Zap, AlertTriangle, Activity, Gauge } from 'lucide-react';
import { Card } from './ui/card';

interface ParameterCardProps {
  label: string;
  value: string | number;
  unit: string;
  icon?: 'brake' | 'acceleration' | 'throttle' | 'alertness';
  status?: 'good' | 'warning' | 'critical';
}

export default function ParameterCard({ label, value, unit, icon, status }: ParameterCardProps) {
  const getIcon = () => {
    switch (icon) {
      case 'brake':
        return <AlertTriangle className="w-5 h-5 text-blue-400" />;
      case 'acceleration':
        return <Zap className="w-5 h-5 text-blue-400" />;
      case 'throttle':
        return <Gauge className="w-5 h-5 text-blue-400" />;
      case 'alertness':
        return <Activity className="w-5 h-5 text-blue-400" />;
      default:
        return <Activity className="w-5 h-5 text-blue-400" />;
    }
  };

  const getStatusColor = () => {
    if (!status) return 'bg-slate-900/50 border-slate-700';
    switch (status) {
      case 'good':
        return 'bg-green-950/30 border-green-600/50';
      case 'warning':
        return 'bg-orange-950/30 border-orange-600/50';
      case 'critical':
        return 'bg-red-950/30 border-red-600/50';
      default:
        return 'bg-slate-900/50 border-slate-700';
    }
  };

  const getValueColor = () => {
    if (!status) return 'text-white';
    switch (status) {
      case 'good':
        return 'text-green-400';
      case 'warning':
        return 'text-orange-400';
      case 'critical':
        return 'text-red-400';
      default:
        return 'text-white';
    }
  };

  return (
    <Card className={`p-4 ${getStatusColor()}`}>
      <div className="flex items-center gap-2 mb-3">
        {getIcon()}
        <p className="text-slate-400 text-sm">{label}</p>
      </div>
      <div className="flex items-baseline gap-1 text-[20px]">
        <p className={`${getValueColor()}`}>{value}</p>
        {unit && <span className="text-slate-400 text-sm text-[16px]">{unit}</span>}
      </div>
    </Card>
  );
}
