import { Card } from './ui/card';

interface ParameterCardProps {
  title: string;
  value: string | number;
  unit: string;
  status?: 'normal' | 'warning' | 'green' | 'yellow' | 'red' | 'none';
}

export function ParameterCard({ title, value, unit, status = 'normal' }: ParameterCardProps) {
  const getStatusColor = () => {
    switch (status) {
      case 'green':
        return 'text-green-400 bg-green-950/30 border-green-600/50';
      case 'yellow':
        return 'text-yellow-400 bg-yellow-950/30 border-yellow-600/50';
      case 'red':
        return 'text-red-400 bg-red-950/30 border-red-600/50';
      case 'warning':
        return 'text-orange-400 bg-orange-950/30 border-orange-600/50';
      case 'none':
        return 'text-slate-400 bg-slate-800/50 border-slate-700';
      default:
        return 'text-blue-400 bg-slate-800/50 border-slate-700';
    }
  };

  const getSignalIndicator = () => {
    if (status === 'green' || status === 'yellow' || status === 'red') {
      return (
        <div className="flex items-center gap-2 mb-2">
          <div className={`w-3 h-3 rounded-full ${
            status === 'green' ? 'bg-green-500' : 
            status === 'yellow' ? 'bg-yellow-500' : 
            'bg-red-500'
          } animate-pulse`}></div>
          <span className="text-xs text-slate-400">Traffic Signal</span>
        </div>
      );
    }
    return null;
  };

  return (
    <Card className={`p-6 ${getStatusColor()} transition-all duration-300`}>
      <div className="space-y-2">
        {getSignalIndicator()}
        <p className="text-slate-400 text-sm">{title}</p>
        <div className="flex items-baseline gap-2">
          <span className="text-3xl text-white">{value}</span>
          {unit && <span className="text-slate-400">{unit}</span>}
        </div>
      </div>
    </Card>
  );
}
