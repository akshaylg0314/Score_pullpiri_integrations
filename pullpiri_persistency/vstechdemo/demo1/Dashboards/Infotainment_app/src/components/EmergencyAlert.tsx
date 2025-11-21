import { AlertTriangle } from 'lucide-react';

interface EmergencyAlertProps {
  mode: 'manual' | 'emergency' | 'autonomous';
}

export function EmergencyAlert({ mode }: EmergencyAlertProps) {
  // Only show for emergency mode
  if (mode !== 'emergency') return null;
  
  return (
    <div className="fixed inset-0 z-50 pointer-events-none">
      {/* Blinking Red Overlay - continuous animation */}
      <div 
        className="absolute inset-0 bg-red-600/30 animate-pulse"
        style={{
          animation: 'pulse 1s cubic-bezier(0.4, 0, 0.6, 1) infinite'
        }}
      />
      
      {/* Alert Modal */}
      <div className="absolute inset-0 flex items-center justify-center p-6">
        <div 
          className="pointer-events-auto max-w-2xl w-full p-8 bg-red-950/95 border-red-600 border-4 rounded-lg shadow-2xl"
          style={{
            animation: 'slideDown 0.5s ease-out'
          }}
        >
          <div className="flex items-start gap-6">
            {/* Icon */}
            <div className="p-4 bg-red-600 rounded-full animate-pulse">
              <AlertTriangle className="w-12 h-12 text-white" />
            </div>
            
            {/* Content */}
            <div className="flex-1">
              <h2 className="text-white mb-2 text-4xl">
                EMERGENCY MODE ACTIVATED
              </h2>
              <p className="text-slate-200 mb-4">
                Vehicle control has been switched to emergency mode. Please take immediate action.
              </p>
              
              {/* Warning Messages */}
              <div className="space-y-2">
                <div className="flex items-center gap-2">
                  <div className="w-2 h-2 rounded-full bg-red-400 animate-pulse" />
                  <p className="text-slate-300 text-sm">
                    All autonomous systems disabled
                  </p>
                </div>
                <div className="flex items-center gap-2">
                  <div className="w-2 h-2 rounded-full bg-red-400 animate-pulse" />
                  <p className="text-slate-300 text-sm">
                    Emergency protocols in effect
                  </p>
                </div>
                <div className="flex items-center gap-2">
                  <div className="w-2 h-2 rounded-full bg-red-400 animate-pulse" />
                  <p className="text-slate-300 text-sm">
                    Emergency services have been notified
                  </p>
                </div>
              </div>
            </div>
          </div>
          
          {/* Bottom Bar */}
          <div className="mt-6 pt-4 border-t border-red-600/50">
            <p className="text-slate-400 text-sm text-center">
              This alert will remain active until emergency mode is cleared
            </p>
          </div>
        </div>
      </div>

      <style>{`
        @keyframes slideDown {
          from {
            opacity: 0;
            transform: translateY(-50px) scale(0.95);
          }
          to {
            opacity: 1;
            transform: translateY(0) scale(1);
          }
        }
      `}</style>
    </div>
  );
}