import React from 'react';
import { Layers, AlertTriangle, RotateCw, CheckCircle2 } from 'lucide-react';
import { DeadLetterTask } from '../types';

interface QueuesPageProps {
  dlq: DeadLetterTask[];
  onResolveDlq: (id: string) => void;
}

export const QueuesPage: React.FC<QueuesPageProps> = ({ dlq, onResolveDlq }) => {
  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-xl font-extrabold tracking-tight text-white flex items-center space-x-2">
          <Layers className="w-5 h-5 text-cyan-400" />
          <span>Queues & Dead Letter Subsystem</span>
        </h1>
        <p className="text-xs text-slate-400 mt-1">
          NATS JetStream dispatch channels, backpressure gauges, and Dead Letter Queue (DLQ) task recovery.
        </p>
      </div>

      {/* Queue Metrics Summary */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="p-4 rounded-2xl bg-slate-900/70 border border-slate-800 space-y-2">
          <div className="text-xs font-semibold text-slate-400 uppercase tracking-wider">Queue Depth</div>
          <div className="text-2xl font-black text-white font-mono">8</div>
          <div className="text-[11px] text-emerald-400 font-mono">Normal backpressure (latency &lt; 5ms)</div>
        </div>

        <div className="p-4 rounded-2xl bg-slate-900/70 border border-slate-800 space-y-2">
          <div className="text-xs font-semibold text-slate-400 uppercase tracking-wider">Active Pull Consumers</div>
          <div className="text-2xl font-black text-cyan-400 font-mono">6</div>
          <div className="text-[11px] text-slate-400 font-mono">Auto-scaled durable workers</div>
        </div>

        <div className="p-4 rounded-2xl bg-slate-900/70 border border-slate-800 space-y-2">
          <div className="text-xs font-semibold text-slate-400 uppercase tracking-wider">DLQ Items</div>
          <div className="text-2xl font-black text-rose-400 font-mono">{dlq.length}</div>
          <div className="text-[11px] text-slate-400 font-mono">Requires operator inspection</div>
        </div>
      </div>

      {/* Dead Letter Queue Inspector Table */}
      <div className="rounded-2xl bg-slate-900/70 border border-slate-800 overflow-hidden">
        <div className="px-5 py-4 border-b border-slate-800 flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <AlertTriangle className="w-4 h-4 text-rose-400" />
            <h2 className="text-sm font-bold text-white uppercase tracking-wider font-mono">
              Dead Letter Queue (DLQ)
            </h2>
          </div>
          <span className="text-xs text-slate-400 font-mono">{dlq.length} unresolved tasks</span>
        </div>

        <table className="w-full text-left text-xs font-mono">
          <thead className="bg-slate-950/80 border-b border-slate-800 text-slate-400 font-bold uppercase tracking-wider text-[11px]">
            <tr>
              <th className="px-5 py-3">Task ID / Run ID</th>
              <th className="px-5 py-3">Failure Reason</th>
              <th className="px-5 py-3">Attempts</th>
              <th className="px-5 py-3">Last Error</th>
              <th className="px-5 py-3 text-right">Recovery Actions</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-800/60 text-slate-300">
            {dlq.map((item) => (
              <tr key={item.id} className="hover:bg-slate-800/40 transition-colors">
                <td className="px-5 py-4">
                  <div className="font-bold text-white text-sm">{item.task_id}</div>
                  <div className="text-[11px] text-slate-500 mt-0.5">{item.workflow_run_id}</div>
                </td>
                <td className="px-5 py-4">
                  <span className="px-2 py-0.5 rounded bg-rose-500/10 text-rose-400 border border-rose-500/20 text-[10px] font-bold">
                    {item.failure_reason}
                  </span>
                </td>
                <td className="px-5 py-4 text-slate-300 font-bold">{item.total_attempts} attempts</td>
                <td className="px-5 py-4 text-slate-400 max-w-xs truncate">{item.last_error || 'Execution failed'}</td>
                <td className="px-5 py-4 text-right">
                  <button
                    onClick={() => onResolveDlq(item.id)}
                    className="flex items-center space-x-1.5 ml-auto px-3 py-1.5 rounded-lg bg-cyan-500/10 hover:bg-cyan-500/20 text-cyan-400 border border-cyan-500/30 text-xs font-bold transition-all"
                  >
                    <RotateCw className="w-3.5 h-3.5" />
                    <span>Requeue & Retry</span>
                  </button>
                </td>
              </tr>
            ))}
            {dlq.length === 0 && (
              <tr>
                <td colSpan={5} className="px-5 py-8 text-center text-slate-500">
                  <CheckCircle2 className="w-6 h-6 text-emerald-400 mx-auto mb-2" />
                  No dead letter tasks. All workloads executing normally.
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
};
