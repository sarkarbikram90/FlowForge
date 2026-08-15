import React, { useState } from 'react';
import {
  GitBranch,
  Search,
  Plus,
  PlayCircle,
  Clock,
  Layers,
  ArrowRight,
  CheckCircle2,
  FileCode,
} from 'lucide-react';
import { Workflow } from '../types';
import { DagViewer } from '../components/DagViewer';

interface WorkflowsPageProps {
  workflows: Workflow[];
  onTriggerRun: (workflowName: string) => void;
  onOpenApplyModal: () => void;
}

export const WorkflowsPage: React.FC<WorkflowsPageProps> = ({
  workflows,
  onTriggerRun,
  onOpenApplyModal,
}) => {
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedWorkflow, setSelectedWorkflow] = useState<Workflow | null>(
    workflows[0] || null
  );

  const filtered = workflows.filter(
    (w) =>
      w.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
      w.description?.toLowerCase().includes(searchTerm.toLowerCase())
  );

  // Sample DAG representation for preview
  const sampleNodes = [
    { id: 'extract-users', name: 'Extract Users', type: 'shell', dependsOn: [] },
    { id: 'extract-orders', name: 'Extract Orders', type: 'shell', dependsOn: [] },
    {
      id: 'transform-data',
      name: 'Transform Data',
      type: 'container',
      dependsOn: ['extract-users', 'extract-orders'],
    },
    { id: 'load-warehouse', name: 'Load Warehouse', type: 'http', dependsOn: ['transform-data'] },
  ];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-extrabold tracking-tight text-white flex items-center space-x-2">
            <GitBranch className="w-5 h-5 text-cyan-400" />
            <span>Workflow Definitions</span>
          </h1>
          <p className="text-xs text-slate-400 mt-1">
            Immutable DAG workflow blueprints, version history, and execution triggers.
          </p>
        </div>

        <div className="flex items-center space-x-3">
          <div className="relative">
            <Search className="w-3.5 h-3.5 text-slate-500 absolute left-3 top-1/2 transform -translate-y-1/2" />
            <input
              type="text"
              placeholder="Search workflows..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="bg-slate-900 border border-slate-800 rounded-lg pl-9 pr-3 py-1.5 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-cyan-500 w-56 font-mono"
            />
          </div>

          <button
            onClick={onOpenApplyModal}
            className="flex items-center space-x-2 bg-cyan-500 hover:bg-cyan-400 text-slate-950 text-xs font-bold px-3 py-1.5 rounded-lg shadow-md shadow-cyan-500/20 transition-all"
          >
            <Plus className="w-3.5 h-3.5 fill-current" />
            <span>New Workflow</span>
          </button>
        </div>
      </div>

      {/* Grid Layout: Workflow List + Visual DAG Preview */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
        {/* Left Column: Workflow List */}
        <div className="lg:col-span-5 space-y-3">
          {filtered.map((wf) => {
            const isSelected = selectedWorkflow?.id === wf.id;

            return (
              <div
                key={wf.id}
                onClick={() => setSelectedWorkflow(wf)}
                className={`p-4 rounded-2xl border transition-all cursor-pointer ${
                  isSelected
                    ? 'bg-slate-900/90 border-cyan-500/40 shadow-md shadow-cyan-500/10'
                    : 'bg-slate-900/50 hover:bg-slate-900/80 border-slate-800/80'
                }`}
              >
                <div className="flex items-start justify-between">
                  <div className="space-y-1">
                    <div className="flex items-center space-x-2">
                      <span className="font-mono text-sm font-bold text-white">
                        {wf.name}
                      </span>
                      <span className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                        active
                      </span>
                    </div>
                    <p className="text-xs text-slate-400 line-clamp-2 leading-relaxed">
                      {wf.description || 'No description provided.'}
                    </p>
                  </div>
                </div>

                <div className="mt-4 pt-3 border-t border-slate-800/80 flex items-center justify-between text-xs">
                  <span className="text-[11px] font-mono text-slate-500">
                    Max Concurrency: <span className="text-slate-300 font-semibold">{wf.concurrency_limit}</span>
                  </span>

                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      onTriggerRun(wf.name);
                    }}
                    className="flex items-center space-x-1.5 text-xs font-bold text-cyan-400 hover:text-cyan-300 font-mono"
                  >
                    <PlayCircle className="w-3.5 h-3.5" />
                    <span>Run Now</span>
                  </button>
                </div>
              </div>
            );
          })}
        </div>

        {/* Right Column: Workflow Detail & DAG Viewer */}
        <div className="lg:col-span-7 space-y-4">
          {selectedWorkflow ? (
            <div className="p-5 rounded-2xl bg-slate-900/70 border border-slate-800 space-y-4">
              <div className="flex items-center justify-between border-b border-slate-800 pb-3">
                <div>
                  <div className="flex items-center space-x-3">
                    <h2 className="text-base font-extrabold text-white font-mono">
                      {selectedWorkflow.name}
                    </h2>
                    <span className="text-[11px] font-mono text-cyan-400 px-2 py-0.5 rounded bg-cyan-500/10 border border-cyan-500/20">
                      Immutable v1
                    </span>
                  </div>
                  <p className="text-xs text-slate-400 mt-1">
                    {selectedWorkflow.description}
                  </p>
                </div>

                <button
                  onClick={() => onTriggerRun(selectedWorkflow.name)}
                  className="flex items-center space-x-2 bg-cyan-500 hover:bg-cyan-400 text-slate-950 text-xs font-bold px-3.5 py-2 rounded-xl shadow-md shadow-cyan-500/20 transition-all"
                >
                  <PlayCircle className="w-4 h-4 fill-current" />
                  <span>Trigger Run</span>
                </button>
              </div>

              {/* Interactive Visual DAG */}
              <div className="space-y-2">
                <div className="flex items-center justify-between text-xs text-slate-400 font-mono">
                  <span>DAG Execution Graph</span>
                  <span className="text-cyan-400">4 Tasks Configured</span>
                </div>
                <DagViewer nodes={sampleNodes} />
              </div>
            </div>
          ) : (
            <div className="h-64 flex items-center justify-center rounded-2xl bg-slate-900/40 border border-slate-800 text-slate-500 text-xs">
              Select a workflow to inspect its DAG structure
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
