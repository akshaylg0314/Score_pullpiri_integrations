interface CircularGaugeProps {
  value: number;
  max: number;
  unit: string;
  color?: 'blue' | 'green' | 'orange' | 'red';
}

export default function CircularGauge({ value, max, unit, color = 'blue' }: CircularGaugeProps) {
  const safeValue = value ?? 0;
  const safeMax = max ?? 100;
  const percentage = Math.min((safeValue / safeMax) * 100, 100);
  const circumference = 2 * Math.PI * 70;
  const strokeDashoffset = circumference - (percentage / 100) * circumference;

  const colorMap = {
    blue: '#3b82f6',
    green: '#22c55e',
    orange: '#f59e0b',
    red: '#ef4444'
  };

  return (
    <div className="relative w-48 h-48">
      <svg className="w-full h-full transform -rotate-90" viewBox="0 0 160 160">
        {/* Background circle */}
        <circle
          cx="80"
          cy="80"
          r="70"
          fill="none"
          stroke="#334155"
          strokeWidth="16"
        />
        {/* Progress circle */}
        <circle
          cx="80"
          cy="80"
          r="70"
          fill="none"
          stroke={colorMap[color]}
          strokeWidth="16"
          strokeDasharray={circumference}
          strokeDashoffset={strokeDashoffset}
          strokeLinecap="round"
          className="transition-all duration-300"
        />
      </svg>
      <div className="absolute inset-0 flex flex-col items-center justify-center">
        <span className="text-white text-[40px]">{safeValue.toFixed(0)}</span>
        <span className="text-slate-400">{unit || ''}</span>
      </div>
    </div>
  );
}
