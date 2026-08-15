import React from 'react';
import {
  LayoutDashboard,
  GitBranch,
  PlayCircle,
  Server,
  Layers,
  Calendar,
  ShieldCheck,
  Settings,
} from 'lucide-react';

export type TabType =
  | 'overview'
  | 'workflows'
  | 'runs'
  | 'workers'
  | 'queues'
  | 'schedules'
  | 'audit'
  | 'settings';

interface SidebarProps {
  activeTab: TabType;
  setActiveTab: (tab: TabType) => void;
  dlqCount?: number;
}

export const Sidebar: React.FC<SidebarProps> = ({
  activeTab,
  setActiveTab,
  dlqCount = 0,
}) => {
  const navItems = [
    { id: 'overview' as TabType, label: 'Overview', icon: LayoutDashboard },
    { id: 'workflows' as TabType, label: 'Workflows', icon: GitBranch },
    { id: 'runs' as TabType, label: 'Runs & Executions', icon: PlayCircle },
    { id: 'workers' as TabType, label: 'Workers Fleet', icon: Server },
    {
      id: 'queues' as TabType,
      label: 'Queues & DLQ',
      icon: Layers,
      badge: dlqCount > 0 ? dlqCount : undefined,
    },
    { id: 'schedules' as TabType, label: 'Schedules & Triggers', icon: Calendar },
    { id: 'audit' as TabType, label: 'Audit Trail', icon: ShieldCheck },
    { id: 'settings' as TabType, label: 'Health & Settings', icon: Settings },
  ];

  return (
    <aside className="w-64 bg-[#0B0F19] border-r border-[#1E293B] flex flex-col justify-between p-4 shrink-0 min-h-[calc(100vh-4rem)]">
      <div className="space-y-1">
        <div className="px-3 py-2 text-[11px] font-bold uppercase tracking-wider text-slate-500 font-mono">
          Platform Operations
        </div>

        {navItems.map((item) => {
          const Icon = item.icon;
          const isActive = activeTab === item.id;

          return (
            <button
              key={item.id}
              onClick={() => setActiveTab(item.id)}
              className={`w-full flex items-center justify-between px-3 py-2.5 rounded-xl text-xs font-medium transition-all ${
                isActive
                  ? 'bg-cyan-500/10 text-cyan-400 font-semibold border border-cyan-500/20 shadow-sm shadow-cyan-500/5'
                  : 'text-slate-400 hover:text-slate-200 hover:bg-slate-900/60'
              }`}
            >
              <div className="flex items-center space-x-3">
                <Icon
                  className={`w-4 h-4 ${
                    isActive ? 'text-cyan-400' : 'text-slate-500'
                  }`}
                />
                <span>{item.label}</span>
              </div>

              {item.badge !== undefined && (
                <span className="px-1.5 py-0.5 text-[10px] font-bold rounded-full bg-rose-500/20 text-rose-400 border border-rose-500/30">
                  {item.badge}
                </span>
              )}
            </button>
          );
        })}
      </div>

      {/* Cluster Node Status footer */}
      <div className="p-3 bg-slate-900/60 rounded-xl border border-slate-800 text-xs space-y-2">
        <div className="flex items-center justify-between text-slate-400">
          <span className="text-[11px] uppercase tracking-wider font-semibold font-mono">Cluster HA</span>
          <span className="flex items-center space-x-1 text-emerald-400 font-mono text-[10px]">
            <span className="w-1.5 h-1.5 rounded-full bg-emerald-400" />
            <span>Optimal</span>
          </span>
        </div>
        <div className="text-[11px] text-slate-500 leading-relaxed font-mono">
          NATS JetStream: Connected<br />
          PostgreSQL: Synchronized
        </div>
      </div>
    </aside>
  );
};
