import React, { useState } from 'react';
import {
  CheckCircle2,
  Clock,
  PlayCircle,
  XCircle,
  RotateCw,
  Ban,
  Terminal,
  Search,
  Filter,
  ArrowLeft,
  Flame,
  Layers,
} from 'lucide-react';
import { TaskRun, WorkflowRun } from '../types';
import { DagViewer } from '../components/DagViewer';

interface RunDetailPageProps {
  runId: string;
  onBack: () => void;
  onCancelRun: (runId: string) => void;
}

export const RunDetailPage: React.FC<RunDetailPageProps> = ({
  runId,
  onBack,
  onCancelRun,
}) => {
  const [logSearch, setLogSearch] = useState('');
  const [logLevel, setLogLevel] = useState('ALL');
  const [selectedTaskId, setSelectedTaskId] = useState<string>('extract-users');

  // Simulated DAG nodes with execution states
  const dagNodes = [
    { id: 'extract-users', name: 'Extract Users', type: 'shell', dependsOn: [], status: 'SUCCEEDED' as any, duration_ms: 1240 },
    { id: 'extract-orders', name: 'Extract Orders', type: 'shell', dependsOn: [], status: 'SUCCEEDED' as any, duration_ms: 980 },
    { id: 'transform-data', name: 'Transform Data', type: 'container', dependsOn: ['extract-users', 'extract-orders'], status: 'RUNNING' as any },
    { id: 'load-warehouse', name: 'Load Warehouse', type: 'http', dependsOn: ['transform-data'], status: 'PENDING' as any },
  ];

  const sampleLogs = [
    { timestamp: '11:04:12.102', level: 'INFO', task: 'extract-users', message: 'Worker "worker-us-east-01" acquired lease token: 7a8b-12cd' },
    { timestamp: '11:04:12.115', level: 'INFO', task: 'extract-users', message: 'Executing shell script: ./scripts/extract_users.sh' },
    { timestamp: '11:04:13.342', level: 'INFO', task: 'extract-users', message: 'Extracted 50,000 records from replica in 1.22s' },
    { timestamp: '11:04:13.355', level: 'INFO', task: 'extract-users', message: 'Task "extract-users" finalized with status SUCCEEDED (duration: 1240ms)' },
    { timestamp: '11:04:13.410', level: 'INFO', task: 'extract-orders', message: 'Worker "worker-us-east-02" starting task "extract-orders"' },
    { timestamp: '11:04:14.390', level: 'INFO', task: 'extract-orders', message: 'Order records batch extracted (980ms)' },
    { timestamp: '11:04:14.420', level: 'INFO', task: 'transform-data', message: 'Prerequisite dependencies met. Dispatched to NATS JetStream queue' },
    { timestamp: '11:04:14.480', level: 'INFO', task: 'transform-data', message: 'Container runner spawning docker image: company/transform:2.4.0' },
    { timestamp: '11:04:15.110', level: 'DEBUG', task: 'transform-data', message: 'Streaming progress: 42% normalized and formatted' },
  ];

  const filteredLogs = sampleLogs.filter((log) => {
    const matchesSearch =
      log.message.toLowerCase().includes(logSearch.toLowerCase()) ||
      log.task.toLowerCase().includes(logSearch.toLowerCase());
    const matchesLevel = logLevel === 'ALL' || log.level === logLevel;
    return matchesSearch && matchesLevel;
  });

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-4">
          <button
            onClick={onBack}
            className="p-2 rounded-xl bg-slate-900 border border-slate-800 hover:bg-slate-800 text-slate-300 transition-all"
          >
            <ArrowLeft className="w-4 h-4" />
          </button>
          <div>
            <div className="flex items-center space-x-3">
              <h1 className="text-xl font-extrabold text-white font-mono">{runId}</h1>
              <span className="text-xs px-2 py-0.5 rounded-full font-mono font-bold bg-amber-500/10 text-amber-400 border border-amber-500/20 animate-pulse">
                RUNNING
              </span>
            </div>
            <p className="text-xs text-slate-400 mt-1 font-mono">
              Workflow: <span className="text-white font-bold">daily-etl-pipeline</span> (Version 1) • Triggered by: cron (0 * * * *)
            </p>
          </div>
        </div>

        <div className="flex items-center space-x-3">
          <button
            onClick={() => onCancelRun(runId)}
            className="flex items-center space-x-2 bg-rose-500/10 hover:bg-rose-500/20 border border-rose-500/30 text-rose-400 text-xs font-bold px-3.5 py-2 rounded-xl transition-all"
          >
            <Ban className="w-3.5 h-3.5" />
            <span>Cancel Run</span>
          </button>
        </div>
      </div>

      {/* Execution Timeline (Gantt style) */}
      <div className="p-5 rounded-2xl bg-slate-900/70 border border-slate-800 space-y-3">
        <div className="flex items-center justify-between text-xs text-slate-400 font-mono">
          <span className="font-bold text-white uppercase tracking-wider">Execution Timeline</span>
          <span>Elapsed: 42s</span>
        </div>

        <div className="space-y-2 pt-2">
          {dagNodes.map((task) => (
            <div key={task.id} className="flex items-center space-x-4 text-xs font-mono">
              <div className="w-36 text-slate-300 font-semibold truncate">{task.id}</div>
              <div className="flex-1 bg-slate-950/80 h-6 rounded-lg p-0.5 relative overflow-hidden border border-slate-800">
                <div
                  style={{
                    width: task.status === 'SUCCEEDED' ? '100%' : task.status === 'RUNNING' ? '65%' : '0%',
                  }}
                  className={`h-full rounded transition-all ${
                    task.status === 'SUCCEEDED'
                      ? 'bg-emerald-500/40 border border-emerald-400/40'
                      : task.status === 'RUNNING'
                      ? 'bg-amber-500/40 border border-amber-400/40 animate-pulse'
                      : ''
                  }`}
                />
              </div>
              <div className="w-20 text-right text-slate-400">
                {task.duration_ms ? `${task.duration_ms}ms` : task.status === 'RUNNING' ? '42s' : '-'}
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Visual DAG Execution Graph */}
      <div className="p-5 rounded-2xl bg-slate-900/70 border border-slate-800 space-y-2">
        <div className="flex items-center justify-between text-xs text-slate-400 font-mono">
          <span className="font-bold text-white uppercase tracking-wider">Live DAG State</span>
          <span className="text-cyan-400">Nodes dynamically colored by worker execution outcome</span>
        </div>
        <DagViewer
          nodes={dagNodes}
          selectedNodeId={selectedTaskId}
          onSelectNode={(id) => setSelectedTaskId(id)}
        />
      </div>

      {/* Real-Time Live Streaming Terminal Log Viewer */}
      <div className="p-5 rounded-2xl bg-[#060910] border border-slate-800 space-y-3 font-mono">
        <div className="flex flex-wrap items-center justify-between gap-3 border-b border-slate-800/80 pb-3">
          <div className="flex items-center space-x-2">
            <Terminal className="w-4 h-4 text-cyan-400" />
            <span className="text-xs font-bold text-white uppercase tracking-wider">
              Execution Logs (Live Tail)
            </span>
          </div>

          <div className="flex items-center space-x-3">
            {/* Search */}
            <div className="relative">
              <Search className="w-3.5 h-3.5 text-slate-500 absolute left-2.5 top-1/2 transform -translate-y-1/2" />
              <input
                type="text"
                placeholder="Filter logs / regex..."
                value={logSearch}
                onChange={(e) => setLogSearch(e.target.value)}
                className="bg-slate-900 border border-slate-800 rounded-lg pl-8 pr-3 py-1 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-cyan-500 w-48"
              />
            </div>

            {/* Level selector */}
            <select
              value={logLevel}
              onChange={(e) => setLogLevel(e.target.value)}
              className="bg-slate-900 border border-slate-800 rounded-lg px-2.5 py-1 text-xs text-slate-300 focus:outline-none focus:border-cyan-500"
            >
              <option value="ALL">ALL LEVELS</option>
              <option value="INFO">INFO</option>
              <option value="DEBUG">DEBUG</option>
              <option value="WARN">WARN</option>
              <option value="ERROR">ERROR</option>
            </select>
          </div>
        </div>

        {/* Terminal logs window */}
        <div className="h-64 overflow-y-auto space-y-1.5 text-xs text-slate-300 pr-2">
          {filteredLogs.map((log, i) => (
            <div key={i} className="flex items-start space-x-3 hover:bg-slate-900/40 p-1 rounded">
              <span className="text-slate-500 text-[11px] shrink-0">{log.timestamp}</span>
              <span
                className={`text-[10px] font-bold px-1.5 py-0.2 rounded shrink-0 ${
                  log.level === 'INFO'
                    ? 'bg-cyan-500/10 text-cyan-400'
                    : log.level === 'DEBUG'
                    ? 'bg-slate-800 text-slate-400'
                    : 'bg-rose-500/10 text-rose-400'
                }`}
              >
                {log.level}
              </span>
              <span className="text-slate-400 text-[11px] shrink-0 font-semibold">[{log.task}]</span>
              <span className="text-slate-200 break-all">{log.message}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
