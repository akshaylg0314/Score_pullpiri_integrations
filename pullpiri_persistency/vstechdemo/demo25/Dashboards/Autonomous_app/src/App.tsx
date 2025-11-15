import { ADDashboard } from './components/ADDashboard';
import { Car } from 'lucide-react';

export default function App() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-slate-800 to-slate-900">
      <div className="container mx-auto p-6">
        {/* Header */}
        <div className="flex items-center gap-3 mb-8">
          <div className="p-3 bg-blue-600 rounded-lg">
            <Car className="w-8 h-8 text-white" />
          </div>
          <div>
            <h1 className="text-white text-3xl">SDV Project Orchestrator</h1>
            <p className="text-slate-400">Autonomous Driving Control System</p>
          </div>
        </div>

        {/* AD Dashboard */}
        <ADDashboard />
      </div>
    </div>
  );
}