import React, { useEffect, useState } from 'react';
import { Navbar } from './components/Navbar';
import { Sidebar, TabType } from './components/Sidebar';
import { OverviewPage } from './pages/OverviewPage';
import { WorkflowsPage } from './pages/WorkflowsPage';
import { RunDetailPage } from './pages/RunDetailPage';
import { WorkersPage } from './pages/WorkersPage';
import { QueuesPage } from './pages/QueuesPage';
import { SchedulesPage } from './pages/SchedulesPage';
import { AuditPage } from './pages/AuditPage';
import { SettingsPage } from './pages/SettingsPage';
import { api } from './api';
import { AuditLog, DeadLetterTask, SystemStats, WorkerRegistration, Workflow, WorkflowRun } from './types';
import { CheckCircle2, PlayCircle, Plus, X, Zap } from 'lucide-react';

export const App: React.FC = () => {
  const [activeTab, setActiveTab] = useState<TabType>('overview');
  const [selectedRunId, setSelectedRunId] = useState<string | null>(null);

  const [stats, setStats] = useState<SystemStats | null>(null);
  const [workflows, setWorkflows] = useState<Workflow[]>([]);
  const [runs, setRuns] = useState<WorkflowRun[]>([]);
  const [workers, setWorkers] = useState<WorkerRegistration[]>([]);
  const [dlq, setDlq] = useState<DeadLetterTask[]>([]);
  const [auditLogs, setAuditLogs] = useState<AuditLog[]>([]);

  // Modals
  const [isApplyModalOpen, setIsApplyModalOpen] = useState(false);
  const [isTriggerModalOpen, setIsTriggerModalOpen] = useState(false);
  const [yamlInput, setYamlInput] = useState(`apiVersion: flowforge.io/v1
kind: Workflow
metadata:
  name: customer-etl
  description: Extract and transform customer data
spec:
  tasks:
    - id: extract
      type: shell
      command: echo "Extracting data..."
    - id: transform
      type: wait
      waitSecs: 2
      dependsOn:
        - extract
    - id: load
      type: shell
      command: echo "Loaded data successfully"
      dependsOn:
        - transform`);
  const [triggerWorkflowName, setTriggerWorkflowName] = useState('');
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  const showToast = (msg: string) => {
    setToastMessage(msg);
    setTimeout(() => setToastMessage(null), 3000);
  };

  const loadData = async () => {
    try {
      const [s, w, r, wrk, d, a] = await Promise.all([
        api.getStats(),
        api.getWorkflows(),
        api.getRuns(),
        api.getWorkers(),
        api.getDlq(),
        api.getAuditLogs(),
      ]);
      setStats(s);
      setWorkflows(w);
      setRuns(r);
      setWorkers(wrk);
      setDlq(d);
      setAuditLogs(a);
    } catch (e) {
      console.error('Failed to load initial data', e);
    }
  };

  useEffect(() => {
    loadData();
    const interval = setInterval(loadData, 5000);
    return () => clearInterval(interval);
  }, []);

  const handleApplyWorkflow = async () => {
    try {
      await api.applyWorkflow(yamlInput);
      setIsApplyModalOpen(false);
      showToast('Workflow blueprint applied successfully!');
      loadData();
    } catch (e: any) {
      alert(`Validation error: ${e.message}`);
    }
  };

  const handleTriggerRun = async (workflowName: string) => {
    try {
      const run = await api.triggerRun(workflowName);
      setIsTriggerModalOpen(false);
      showToast(`Run ${run.id} triggered!`);
      loadData();
    } catch (e: any) {
      alert(`Trigger failed: ${e.message}`);
    }
  };

  const handleCancelRun = async (runId: string) => {
    try {
      await api.cancelRun(runId);
      showToast(`Run ${runId} canceled`);
      loadData();
    } catch (e: any) {
      alert(`Cancel failed: ${e.message}`);
    }
  };

  const handleDrainWorker = async (workerId: string) => {
    try {
      await api.drainWorker(workerId);
      showToast(`Worker ${workerId} set to DRAINING`);
      loadData();
    } catch (e: any) {
      alert(`Drain failed: ${e.message}`);
    }
  };

  const handleResolveDlq = async (id: string) => {
    try {
      await api.resolveDlq(id);
      showToast(`DLQ task resolved and requeued`);
      loadData();
    } catch (e: any) {
      alert(`Resolve failed: ${e.message}`);
    }
  };

  return (
    <div className="min-h-screen bg-[#0B0E17] text-slate-100 flex flex-col font-sans selection:bg-cyan-500/20 selection:text-cyan-300">
      {/* Toast Notification */}
      {toastMessage && (
        <div className="fixed bottom-6 right-6 z-50 flex items-center space-x-2 bg-slate-900 border border-cyan-500/40 text-cyan-300 px-4 py-3 rounded-2xl shadow-xl shadow-black/50 text-xs font-mono font-bold animate-bounce">
          <CheckCircle2 className="w-4 h-4 text-cyan-400" />
          <span>{toastMessage}</span>
        </div>
      )}

      {/* Top Navbar */}
      <Navbar
        stats={stats}
        onOpenApplyModal={() => setIsApplyModalOpen(true)}
        onOpenTriggerModal={() => {
          setTriggerWorkflowName(workflows[0]?.name || 'daily-etl-pipeline');
          setIsTriggerModalOpen(true);
        }}
      />

      {/* Main App Layout: Sidebar + Content */}
      <div className="flex-1 flex">
        <Sidebar
          activeTab={activeTab}
          setActiveTab={(tab) => {
            setActiveTab(tab);
            setSelectedRunId(null);
          }}
          dlqCount={dlq.length}
        />

        <main className="flex-1 p-8 max-w-7xl mx-auto overflow-y-auto">
          {selectedRunId ? (
            <RunDetailPage
              runId={selectedRunId}
              onBack={() => setSelectedRunId(null)}
              onCancelRun={handleCancelRun}
            />
          ) : (
            <>
              {activeTab === 'overview' && (
                <OverviewPage
                  stats={stats}
                  runs={runs}
                  onSelectRun={(id) => setSelectedRunId(id)}
                  onNavigateToTab={(tab) => setActiveTab(tab)}
                />
              )}

              {activeTab === 'workflows' && (
                <WorkflowsPage
                  workflows={workflows}
                  onTriggerRun={(name) => handleTriggerRun(name)}
                  onOpenApplyModal={() => setIsApplyModalOpen(true)}
                />
              )}

              {activeTab === 'runs' && (
                <div className="space-y-6">
                  <div>
                    <h1 className="text-xl font-extrabold tracking-tight text-white flex items-center space-x-2">
                      <PlayCircle className="w-5 h-5 text-cyan-400" />
                      <span>Execution Runs</span>
                    </h1>
                    <p className="text-xs text-slate-400 mt-1">
                      Historical and live workflow execution runs, status transitions, and durations.
                    </p>
                  </div>

                  <div className="rounded-2xl bg-slate-900/70 border border-slate-800 overflow-hidden">
                    <table className="w-full text-left text-xs font-mono">
                      <thead className="bg-slate-950/80 border-b border-slate-800 text-slate-400 font-bold uppercase tracking-wider text-[11px]">
                        <tr>
                          <th className="px-5 py-3">Run ID</th>
                          <th className="px-5 py-3">Status</th>
                          <th className="px-5 py-3">Triggered By</th>
                          <th className="px-5 py-3">Started At</th>
                          <th className="px-5 py-3">Duration</th>
                          <th className="px-5 py-3 text-right">Actions</th>
                        </tr>
                      </thead>
                      <tbody className="divide-y divide-slate-800/60 text-slate-300">
                        {runs.map((run) => (
                          <tr
                            key={run.id}
                            onClick={() => setSelectedRunId(run.id)}
                            className="hover:bg-slate-800/40 transition-colors cursor-pointer"
                          >
                            <td className="px-5 py-4 font-bold text-white text-sm">{run.id}</td>
                            <td className="px-5 py-4">
                              <span
                                className={`px-2 py-0.5 rounded-full text-[10px] font-bold border ${
                                  run.status === 'SUCCEEDED'
                                    ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'
                                    : run.status === 'RUNNING'
                                    ? 'bg-amber-500/10 text-amber-400 border-amber-500/20 animate-pulse'
                                    : 'bg-rose-500/10 text-rose-400 border-rose-500/20'
                                }`}
                              >
                                {run.status}
                              </span>
                            </td>
                            <td className="px-5 py-4 text-slate-400">{run.triggered_by}</td>
                            <td className="px-5 py-4 text-slate-400">
                              {new Date(run.created_at).toLocaleTimeString()}
                            </td>
                            <td className="px-5 py-4 text-cyan-400 font-semibold">
                              {run.duration_ms ? `${Math.round(run.duration_ms / 1000)}s` : '-'}
                            </td>
                            <td className="px-5 py-4 text-right">
                              <button className="px-3 py-1 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-semibold">
                                Inspect
                              </button>
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </div>
              )}

              {activeTab === 'workers' && (
                <WorkersPage workers={workers} onDrainWorker={handleDrainWorker} />
              )}

              {activeTab === 'queues' && (
                <QueuesPage dlq={dlq} onResolveDlq={handleResolveDlq} />
              )}

              {activeTab === 'schedules' && (
                <SchedulesPage onTriggerWorkflow={handleTriggerRun} />
              )}

              {activeTab === 'audit' && <AuditPage logs={auditLogs} />}

              {activeTab === 'settings' && <SettingsPage />}
            </>
          )}
        </main>
      </div>

      {/* Apply Workflow YAML Modal */}
      {isApplyModalOpen && (
        <div className="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm flex items-center justify-center p-4">
          <div className="w-full max-w-2xl bg-slate-900 rounded-2xl border border-slate-800 shadow-2xl p-6 space-y-4 font-mono">
            <div className="flex items-center justify-between border-b border-slate-800 pb-3">
              <h3 className="text-sm font-bold text-white flex items-center space-x-2">
                <Plus className="w-4 h-4 text-cyan-400" />
                <span>Apply Workflow Definition (YAML)</span>
              </h3>
              <button
                onClick={() => setIsApplyModalOpen(false)}
                className="text-slate-500 hover:text-white"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <textarea
              value={yamlInput}
              onChange={(e) => setYamlInput(e.target.value)}
              rows={12}
              className="w-full bg-[#060910] border border-slate-800 rounded-xl p-3 text-xs text-cyan-300 font-mono focus:outline-none focus:border-cyan-500 resize-none"
            />

            <div className="flex justify-end space-x-3 pt-2">
              <button
                onClick={() => setIsApplyModalOpen(false)}
                className="px-4 py-2 rounded-xl text-xs font-semibold text-slate-400 hover:text-white"
              >
                Cancel
              </button>
              <button
                onClick={handleApplyWorkflow}
                className="px-4 py-2 rounded-xl bg-cyan-500 hover:bg-cyan-400 text-slate-950 text-xs font-bold shadow-md shadow-cyan-500/20"
              >
                Apply & Compile
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Trigger Run Modal */}
      {isTriggerModalOpen && (
        <div className="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm flex items-center justify-center p-4">
          <div className="w-full max-w-md bg-slate-900 rounded-2xl border border-slate-800 shadow-2xl p-6 space-y-4">
            <div className="flex items-center justify-between border-b border-slate-800 pb-3">
              <h3 className="text-sm font-bold text-white flex items-center space-x-2">
                <Zap className="w-4 h-4 text-cyan-400" />
                <span>Trigger Workflow Execution</span>
              </h3>
              <button
                onClick={() => setIsTriggerModalOpen(false)}
                className="text-slate-500 hover:text-white"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <div className="space-y-2">
              <label className="text-xs font-mono text-slate-400">Select Workflow:</label>
              <select
                value={triggerWorkflowName}
                onChange={(e) => setTriggerWorkflowName(e.target.value)}
                className="w-full bg-slate-950 border border-slate-800 rounded-xl p-2.5 text-xs text-white font-mono focus:outline-none focus:border-cyan-500"
              >
                {workflows.map((w) => (
                  <option key={w.id} value={w.name}>
                    {w.name}
                  </option>
                ))}
              </select>
            </div>

            <div className="flex justify-end space-x-3 pt-2">
              <button
                onClick={() => setIsTriggerModalOpen(false)}
                className="px-4 py-2 rounded-xl text-xs font-semibold text-slate-400 hover:text-white"
              >
                Cancel
              </button>
              <button
                onClick={() => handleTriggerRun(triggerWorkflowName)}
                className="px-4 py-2 rounded-xl bg-cyan-500 hover:bg-cyan-400 text-slate-950 text-xs font-bold shadow-md shadow-cyan-500/20"
              >
                Trigger Execution
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
