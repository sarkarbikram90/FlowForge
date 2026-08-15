import React from 'react';
import { Server } from 'lucide-react';
import { WorkerRegistration } from '../types';

interface WorkersPageProps {
  workers: WorkerRegistration[];
  onDrainWorker: (workerId: string) => void;
}

export const WorkersPage: React.FC<WorkersPageProps> = ({
  workers,
  onDrainWorker,
}) => {
  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-extrabold tracking-tight text-white flex items-center space-x-2">
            <Server className="w-5 h-5 text-cyan-400" />
            <span>Worker Fleet Management</span>
          </h1>
          <p className="text-xs text-slate-400 mt-1">
            Distributed execution nodes, capability routing, load capacity, and graceful rolling draining.
          </p>
        </div>
      </div>

      {/* Workers Table */}
      <div className="rounded-2xl bg-slate-900/70 border border-slate-800 overflow-hidden">
        <table className="w-full text-left text-xs font-mono">
          <thead className="bg-slate-950/80 border-b border-slate-800 text-slate-400 font-bold uppercase tracking-wider text-[11px]">
            <tr>
              <th className="px-5 py-3">Worker ID & Hostname</th>
              <th className="px-5 py-3">Status</th>
              <th className="px-5 py-3">Architecture / OS</th>
              <th className="px-5 py-3">Current Load / Capacity</th>
              <th className="px-5 py-3">Capabilities</th>
              <th className="px-5 py-3 text-right">Actions</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-800/60 text-slate-300">
            {workers.map((worker) => {
              const utilPercent = Math.round(
                (worker.current_load / Math.max(1, worker.max_concurrency)) * 100
              );

              return (
                <tr key={worker.worker_id} className="hover:bg-slate-800/40 transition-colors">
                  <td className="px-5 py-4">
                    <div className="font-bold text-white text-sm">{worker.worker_id}</div>
                    <div className="text-[11px] text-slate-500 mt-0.5 truncate max-w-xs">{worker.hostname}</div>
                  </td>
                  <td className="px-5 py-4">
                    <span
                      className={`px-2 py-0.5 rounded-full text-[10px] font-bold border ${
                        worker.status === 'ONLINE'
                          ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'
                          : worker.status === 'DRAINING'
                          ? 'bg-amber-500/10 text-amber-400 border-amber-500/20'
                          : 'bg-rose-500/10 text-rose-400 border-rose-500/20'
                      }`}
                    >
                      {worker.status}
                    </span>
                  </td>
                  <td className="px-5 py-4">
                    <div className="text-slate-200">{worker.os}</div>
                    <div className="text-[11px] text-slate-500">{worker.architecture}</div>
                  </td>
                  <td className="px-5 py-4">
                    <div className="flex items-center space-x-2">
                      <div className="w-28 bg-slate-950 h-3 rounded-full overflow-hidden p-0.5 border border-slate-800">
                        <div
                          style={{ width: `${utilPercent}%` }}
                          className="h-full bg-cyan-400 rounded-full"
                        />
                      </div>
                      <span className="text-[11px] text-cyan-300">
                        {worker.current_load}/{worker.max_concurrency} ({utilPercent}%)
                      </span>
                    </div>
                  </td>
                  <td className="px-5 py-4">
                    <div className="flex flex-wrap gap-1 max-w-xs">
                      {worker.capabilities.map((cap) => (
                        <span
                          key={cap}
                          className="px-1.5 py-0.2 rounded bg-slate-800 text-[10px] text-slate-300 border border-slate-700"
                        >
                          {cap}
                        </span>
                      ))}
                    </div>
                  </td>
                  <td className="px-5 py-4 text-right">
                    {worker.status !== 'DRAINING' && (
                      <button
                        onClick={() => onDrainWorker(worker.worker_id)}
                        className="px-3 py-1 rounded-lg bg-amber-500/10 hover:bg-amber-500/20 text-amber-400 border border-amber-500/30 text-xs font-semibold transition-all"
                      >
                        Drain
                      </button>
                    )}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
};
