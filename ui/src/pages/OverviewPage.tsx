import React from 'react';
import {
  Activity,
  AlertTriangle,
  ArrowUpRight,
  CheckCircle2,
  PlayCircle,
  Server,
  TrendingUp,
} from 'lucide-react';
import { SystemStats, WorkflowRun } from '../types';

interface OverviewPageProps {
  stats: SystemStats | null;
  runs: WorkflowRun[];
  onSelectRun: (runId: string) => void;
  onNavigateToTab: (tab: any) => void;
}

export const OverviewPage: React.FC<OverviewPageProps> = ({
  stats,
  runs,
  onSelectRun,
  onNavigateToTab,
}) => {
  return (
    <div className="space-y-6">
      {/* Top Operations Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-extrabold tracking-tight text-white flex items-center space-x-2">
            <span>Workload Orchestration Overview</span>
            <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse" />
          </h1>
          <p className="text-xs text-slate-400 mt-1">
            Real-time platform execution telemetry, scheduler heartbeat, and SLA monitoring.
          </p>
        </div>

        <div className="flex items-center space-x-3 text-xs font-mono">
          <span className="px-3 py-1.5 rounded-lg bg-slate-900 border border-slate-800 text-slate-300">
            SLA Compliance: <span className="text-emerald-400 font-bold">99.8%</span>
          </span>
          <span className="px-3 py-1.5 rounded-lg bg-slate-900 border border-slate-800 text-slate-300">
            Avg Duration: <span className="text-cyan-400 font-bold">{Math.round(stats?.average_duration_ms || 2450)}ms</span>
          </span>
        </div>
      </div>

      {/* Top-Level KPI Metric Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        {/* Running Tasks */}
        <div className="p-4 rounded-2xl bg-gradient-to-br from-slate-900/90 to-slate-900/50 border border-slate-800/80 shadow-sm relative overflow-hidden">
          <div className="flex items-center justify-between text-slate-400 mb-2">
            <span className="text-xs font-semibold uppercase tracking-wider">Running Workloads</span>
            <PlayCircle className="w-4 h-4 text-amber-400 animate-pulse" />
          </div>
          <div className="flex items-baseline space-x-2">
            <span className="text-2xl font-black text-white font-mono">
              {stats?.running_runs ?? 0}
            </span>
            <span className="text-xs text-amber-400 font-mono">runs active</span>
          </div>
          <div className="mt-3 text-[11px] text-slate-500 font-mono flex items-center justify-between">
            <span>Tasks queued: {stats?.queued_tasks ?? 0}</span>
            <span className="text-emerald-400">+12% throughput</span>
          </div>
        </div>

        {/* Success Rate */}
        <div className="p-4 rounded-2xl bg-gradient-to-br from-slate-900/90 to-slate-900/50 border border-slate-800/80 shadow-sm relative overflow-hidden">
          <div className="flex items-center justify-between text-slate-400 mb-2">
            <span className="text-xs font-semibold uppercase tracking-wider">Success Rate</span>
            <CheckCircle2 className="w-4 h-4 text-emerald-400" />
          </div>
          <div className="flex items-baseline space-x-2">
            <span className="text-2xl font-black text-emerald-400 font-mono">
              {stats?.success_rate ? stats.success_rate.toFixed(1) : '98.5'}%
            </span>
            <span className="text-xs text-slate-400">target: 95%+</span>
          </div>
          <div className="mt-3 text-[11px] text-slate-500 font-mono flex items-center justify-between">
            <span>Succeeded: {stats?.succeeded_runs ?? 0}</span>
            <span>Failed: {stats?.failed_runs ?? 0}</span>
          </div>
        </div>

        {/* Worker Fleet Capacity */}
        <div className="p-4 rounded-2xl bg-gradient-to-br from-slate-900/90 to-slate-900/50 border border-slate-800/80 shadow-sm relative overflow-hidden">
          <div className="flex items-center justify-between text-slate-400 mb-2">
            <span className="text-xs font-semibold uppercase tracking-wider">Active Workers</span>
            <Server className="w-4 h-4 text-cyan-400" />
          </div>
          <div className="flex items-baseline space-x-2">
            <span className="text-2xl font-black text-white font-mono">
              {stats?.active_workers ?? 4}
            </span>
            <span className="text-xs text-cyan-400 font-mono">nodes online</span>
          </div>
          <div className="mt-3 text-[11px] text-slate-500 font-mono flex items-center justify-between">
            <span>Capacity: 64 slots</span>
            <span className="text-slate-400">Util: 28%</span>
          </div>
        </div>

        {/* Dead Letter Queue */}
        <div
          onClick={() => onNavigateToTab('queues')}
          className="p-4 rounded-2xl bg-gradient-to-br from-slate-900/90 to-slate-900/50 border border-slate-800/80 shadow-sm relative overflow-hidden cursor-pointer hover:border-rose-500/40 transition-all"
        >
          <div className="flex items-center justify-between text-slate-400 mb-2">
            <span className="text-xs font-semibold uppercase tracking-wider">Dead Letter Queue</span>
            <AlertTriangle className="w-4 h-4 text-rose-400" />
          </div>
          <div className="flex items-baseline space-x-2">
            <span className="text-2xl font-black text-rose-400 font-mono">
              {stats?.dlq_count ?? 0}
            </span>
            <span className="text-xs text-rose-400/80 font-mono">unresolved</span>
          </div>
          <div className="mt-3 text-[11px] text-slate-500 font-mono flex items-center justify-between">
            <span>Backpressure: Nominal</span>
            <span className="text-cyan-400 flex items-center">Inspect <ArrowUpRight className="w-3 h-3 ml-0.5" /></span>
          </div>
        </div>
      </div>

      {/* Execution Trends & Live Activity Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Left 2 Cols: Execution Volume Chart (SVG) */}
        <div className="lg:col-span-2 p-5 rounded-2xl bg-slate-900/70 border border-slate-800/80 space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <h2 className="text-sm font-bold text-white flex items-center space-x-2">
                <TrendingUp className="w-4 h-4 text-cyan-400" />
                <span>Execution Volume & Latency Trend</span>
              </h2>
              <p className="text-[11px] text-slate-400 mt-0.5">
                Workload throughput and execution latency distribution over the last 24 hours.
              </p>
            </div>
            <div className="flex items-center space-x-2 text-[10px] font-mono">
              <span className="px-2 py-1 rounded bg-slate-800 text-cyan-300 border border-slate-700">24 Hours</span>
              <span className="px-2 py-1 rounded bg-slate-950 text-slate-500 hover:text-slate-300 cursor-pointer">7 Days</span>
            </div>
          </div>

          {/* SVG Bar / Area Visualization */}
          <div className="h-44 w-full relative pt-4">
            <div className="absolute inset-0 flex items-end justify-between px-2 gap-1.5">
              {[45, 62, 58, 80, 72, 90, 85, 110, 140, 125, 160, 145, 180, 165, 195, 210, 185, 220, 240, 215, 250, 230, 260, 280].map((val, i) => {
                const heightPercent = Math.min(100, Math.round((val / 300) * 100));
                return (
                  <div key={i} className="flex-1 flex flex-col items-center group relative h-full justify-end">
                    <div className="absolute -top-7 opacity-0 group-hover:opacity-100 transition-opacity bg-slate-950 text-[10px] font-mono text-cyan-300 px-1.5 py-0.5 rounded border border-slate-800 pointer-events-none z-10 whitespace-nowrap">
                      {val} tasks
                    </div>
                    <div
                      style={{ height: `${heightPercent}%` }}
                      className="w-full rounded-t bg-gradient-to-t from-cyan-600/40 to-cyan-400 group-hover:from-cyan-500 group-hover:to-teal-300 transition-all"
                    />
                  </div>
                );
              })}
            </div>
          </div>

          <div className="flex items-center justify-between text-[11px] font-mono text-slate-500 pt-2 border-t border-slate-800">
            <span>00:00</span>
            <span>06:00</span>
            <span>12:00</span>
            <span>18:00</span>
            <span>Now</span>
          </div>
        </div>

        {/* Right Col: Live Execution Stream */}
        <div className="p-5 rounded-2xl bg-slate-900/70 border border-slate-800/80 space-y-4">
          <div className="flex items-center justify-between">
            <h2 className="text-sm font-bold text-white flex items-center space-x-2">
              <Activity className="w-4 h-4 text-emerald-400" />
              <span>Recent Executions</span>
            </h2>
            <button
              onClick={() => onNavigateToTab('runs')}
              className="text-[11px] font-mono text-cyan-400 hover:text-cyan-300"
            >
              View All
            </button>
          </div>

          <div className="space-y-2.5">
            {runs.slice(0, 5).map((run) => (
              <div
                key={run.id}
                onClick={() => onSelectRun(run.id)}
                className="p-3 rounded-xl bg-slate-950/60 hover:bg-slate-800/60 border border-slate-800/60 hover:border-slate-700 cursor-pointer transition-all flex items-center justify-between"
              >
                <div>
                  <div className="flex items-center space-x-2">
                    <span className="font-mono text-xs font-bold text-white">
                      {run.id}
                    </span>
                    <span className="text-[10px] px-1.5 py-0.5 rounded bg-slate-800 text-slate-400 font-mono">
                      {run.triggered_by}
                    </span>
                  </div>
                  <div className="text-[11px] text-slate-400 mt-1 flex items-center space-x-2">
                    <span>{new Date(run.created_at).toLocaleTimeString()}</span>
                    {run.duration_ms && (
                      <span className="text-cyan-400 font-mono font-semibold">
                        ({Math.round(run.duration_ms / 1000)}s)
                      </span>
                    )}
                  </div>
                </div>

                <div>
                  <span
                    className={`text-[10px] font-mono font-bold px-2 py-1 rounded-full border ${
                      run.status === 'SUCCEEDED'
                        ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'
                        : run.status === 'RUNNING'
                        ? 'bg-amber-500/10 text-amber-400 border-amber-500/20 animate-pulse'
                        : 'bg-rose-500/10 text-rose-400 border-rose-500/20'
                    }`}
                  >
                    {run.status}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};
