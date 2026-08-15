import React from 'react';
import { Calendar, Clock, PlayCircle, Zap, Globe } from 'lucide-react';

interface SchedulesPageProps {
  onTriggerWorkflow: (name: string) => void;
}

export const SchedulesPage: React.FC<SchedulesPageProps> = ({ onTriggerWorkflow }) => {
  const schedules = [
    {
      id: 'sch-1',
      workflowName: 'daily-etl-pipeline',
      cron: '0 * * * *',
      humanDesc: 'Runs every hour at minute 0',
      timezone: 'UTC',
      status: 'ACTIVE',
      nextFire: 'In 24 minutes (12:00:00 UTC)',
      lastFired: '36 minutes ago (11:00:00 UTC)',
    },
    {
      id: 'sch-2',
      workflowName: 'k8s-model-training',
      cron: '0 2 * * *',
      humanDesc: 'Runs daily at 02:00 UTC',
      timezone: 'UTC',
      status: 'ACTIVE',
      nextFire: 'In 14 hours (02:00:00 UTC)',
      lastFired: 'Yesterday at 02:00:00 UTC',
    },
    {
      id: 'sch-3',
      workflowName: 'security-compliance-audit',
      cron: '*/15 * * * *',
      humanDesc: 'Runs every 15 minutes',
      timezone: 'UTC',
      status: 'ACTIVE',
      nextFire: 'In 9 minutes (11:45:00 UTC)',
      lastFired: '6 minutes ago (11:30:00 UTC)',
    },
  ];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-xl font-extrabold tracking-tight text-white flex items-center space-x-2">
          <Calendar className="w-5 h-5 text-cyan-400" />
          <span>Schedules & Event Triggers</span>
        </h1>
        <p className="text-xs text-slate-400 mt-1">
          Automated time-based cron schedules, webhooks, and dependency event routers.
        </p>
      </div>

      {/* Schedules List */}
      <div className="rounded-2xl bg-slate-900/70 border border-slate-800 overflow-hidden">
        <table className="w-full text-left text-xs font-mono">
          <thead className="bg-slate-950/80 border-b border-slate-800 text-slate-400 font-bold uppercase tracking-wider text-[11px]">
            <tr>
              <th className="px-5 py-3">Workflow Target</th>
              <th className="px-5 py-3">Cron Expression</th>
              <th className="px-5 py-3">Human Schedule</th>
              <th className="px-5 py-3">Next Fire Time</th>
              <th className="px-5 py-3">Last Fired</th>
              <th className="px-5 py-3 text-right">Actions</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-800/60 text-slate-300">
            {schedules.map((sch) => (
              <tr key={sch.id} className="hover:bg-slate-800/40 transition-colors">
                <td className="px-5 py-4">
                  <div className="font-bold text-white text-sm">{sch.workflowName}</div>
                  <div className="text-[11px] text-cyan-400 mt-0.5">Timezone: {sch.timezone}</div>
                </td>
                <td className="px-5 py-4">
                  <span className="px-2 py-1 rounded bg-slate-950 text-cyan-300 border border-slate-800 font-bold">
                    {sch.cron}
                  </span>
                </td>
                <td className="px-5 py-4 text-slate-300">{sch.humanDesc}</td>
                <td className="px-5 py-4 text-emerald-400 font-bold">{sch.nextFire}</td>
                <td className="px-5 py-4 text-slate-400">{sch.lastFired}</td>
                <td className="px-5 py-4 text-right">
                  <button
                    onClick={() => onTriggerWorkflow(sch.workflowName)}
                    className="flex items-center space-x-1.5 ml-auto px-3 py-1.5 rounded-lg bg-cyan-500/10 hover:bg-cyan-500/20 text-cyan-400 border border-cyan-500/30 text-xs font-bold transition-all"
                  >
                    <PlayCircle className="w-3.5 h-3.5" />
                    <span>Trigger Now</span>
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
};
