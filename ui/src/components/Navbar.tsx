import React from 'react';
import {
  Activity,
  Layers,
  Search,
  Plus,
  Radio,
  Server,
  Zap,
} from 'lucide-react';
import { SystemStats } from '../types';

interface NavbarProps {
  stats: SystemStats | null;
  onOpenApplyModal: () => void;
  onOpenTriggerModal: () => void;
}

export const Navbar: React.FC<NavbarProps> = ({
  stats,
  onOpenApplyModal,
  onOpenTriggerModal,
}) => {
  return (
    <header className="h-16 bg-[#0E131F]/90 backdrop-blur-md border-b border-[#1E293B] px-6 flex items-center justify-between sticky top-0 z-40">
      {/* Brand & Organization */}
      <div className="flex items-center space-x-6">
        <div className="flex items-center space-x-3">
          <div className="w-9 h-9 rounded-xl bg-gradient-to-tr from-cyan-600 to-teal-400 flex items-center justify-center shadow-lg shadow-cyan-500/20">
            <Zap className="w-5 h-5 text-slate-950 fill-current" />
          </div>
          <div>
            <div className="flex items-center space-x-2">
              <span className="font-extrabold text-lg tracking-tight bg-gradient-to-r from-white via-slate-200 to-slate-400 bg-clip-text text-transparent">
                FlowForge
              </span>
              <span className="text-[10px] px-1.5 py-0.5 rounded bg-cyan-500/10 text-cyan-400 font-mono font-semibold border border-cyan-500/20">
                v0.2.0
              </span>
            </div>
          </div>
        </div>

        <div className="h-5 w-[1px] bg-slate-800" />

        {/* Workspace / Tenancy Selector */}
        <div className="flex items-center space-x-2 bg-slate-900/80 px-3 py-1.5 rounded-lg border border-slate-800 text-xs text-slate-300">
          <Layers className="w-3.5 h-3.5 text-cyan-400" />
          <span className="text-slate-400">Org:</span>
          <span className="font-semibold text-white">FlowForge Global</span>
          <span className="text-slate-600">/</span>
          <span className="text-slate-400">Project:</span>
          <span className="font-semibold text-cyan-300">Production</span>
        </div>
      </div>

      {/* Center Quick Stats */}
      <div className="hidden md:flex items-center space-x-6 text-xs text-slate-400 font-mono">
        <div className="flex items-center space-x-2">
          <div className="w-2 h-2 rounded-full bg-emerald-400 animate-ping" />
          <span className="text-slate-300">HA Leader:</span>
          <span className="text-emerald-400 font-semibold">{stats?.scheduler_leader_id || 'sched-primary'}</span>
        </div>

        <div className="flex items-center space-x-2">
          <Server className="w-3.5 h-3.5 text-slate-500" />
          <span>Workers:</span>
          <span className="text-white font-bold">{stats?.active_workers ?? 4}</span>
        </div>

        <div className="flex items-center space-x-2">
          <Activity className="w-3.5 h-3.5 text-cyan-400" />
          <span>Active Tasks:</span>
          <span className="text-cyan-400 font-bold">{stats?.running_tasks ?? 0}</span>
        </div>
      </div>

      {/* Action CTA Buttons */}
      <div className="flex items-center space-x-3">
        <button
          onClick={onOpenApplyModal}
          className="flex items-center space-x-2 bg-slate-900 hover:bg-slate-800 border border-slate-700 hover:border-slate-600 text-slate-200 text-xs font-semibold px-3 py-2 rounded-lg transition-all"
        >
          <Plus className="w-3.5 h-3.5 text-cyan-400" />
          <span>Apply Workflow</span>
        </button>

        <button
          onClick={onOpenTriggerModal}
          className="flex items-center space-x-2 bg-gradient-to-r from-cyan-500 to-teal-500 hover:from-cyan-400 hover:to-teal-400 text-slate-950 text-xs font-bold px-3.5 py-2 rounded-lg shadow-md shadow-cyan-500/20 transition-all"
        >
          <Zap className="w-3.5 h-3.5 fill-current" />
          <span>Trigger Run</span>
        </button>
      </div>
    </header>
  );
};
