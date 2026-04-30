import React, { useState, useEffect, useCallback } from 'react';

const API = '/api/v1';

function useFetch(url, interval = 5000) {
  const [data, setData] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  const fetchData = useCallback(async () => {
    try {
      const res = await fetch(url);
      const json = await res.json();
      setData(json.data);
      setError(null);
    } catch (e) {
      setError(e.message);
    } finally {
      setLoading(false);
    }
  }, [url]);

  useEffect(() => {
    fetchData();
    const id = setInterval(fetchData, interval);
    return () => clearInterval(id);
  }, [fetchData, interval]);

  return { data, loading, error, refetch: fetchData };
}

function StatusBadge({ status }) {
  const cls = {
    success: 'badge-success', running: 'badge-running',
    failed: 'badge-failed', pending: 'badge-pending',
    queued: 'badge-queued', retrying: 'badge-running',
    cancelled: 'badge-failed', skipped: 'badge-queued',
  }[status] || 'badge-queued';
  return <span className={`badge ${cls}`}>{status}</span>;
}

function Sidebar({ page, setPage }) {
  const items = [
    { id: 'dashboard', icon: '📊', label: 'Dashboard' },
    { id: 'dags', icon: '🔀', label: 'DAGs' },
    { id: 'runs', icon: '▶️', label: 'Runs' },
    { id: 'workers', icon: '⚙️', label: 'Workers' },
  ];
  return (
    <div className="sidebar">
      <div className="sidebar-logo"><span>⚡</span> FlowForge</div>
      <div className="sidebar-subtitle">Workflow Scheduler</div>
      {items.map(it => (
        <div key={it.id} className={`nav-item ${page === it.id ? 'active' : ''}`}
          onClick={() => setPage(it.id)}>
          <span>{it.icon}</span><span>{it.label}</span>
        </div>
      ))}
    </div>
  );
}

function Dashboard() {
  const { data: status, loading } = useFetch(`${API}/status`);
  const { data: runs } = useFetch(`${API}/runs`);

  if (loading) return <div className="loading"><div className="spinner" /></div>;

  const recentRuns = (runs || []).slice(0, 5);
  const stats = status || { active_dags: 0, total_runs: 0, running_tasks: 0, active_workers: 0 };

  return (
    <>
      <div className="page-header">
        <h1 className="page-title">Dashboard</h1>
      </div>
      <div className="card-grid">
        {[
          { label: 'Active DAGs', value: stats.active_dags, icon: '🔀' },
          { label: 'Total Runs', value: stats.total_runs, icon: '▶️' },
          { label: 'Active Tasks', value: stats.running_tasks, icon: '⚡' },
          { label: 'Workers Online', value: stats.active_workers, icon: '⚙️' },
        ].map(s => (
          <div key={s.label} className="card stat-card">
            <div className="stat-label">{s.icon} {s.label}</div>
            <div className="stat-value">{s.value}</div>
          </div>
        ))}
      </div>
      <h2 style={{ fontSize: '1.1rem', marginBottom: 16, color: 'var(--text-secondary)' }}>Recent Runs</h2>
      <div className="table-container">
        <table>
          <thead><tr><th>Run ID</th><th>DAG</th><th>Status</th><th>Triggered By</th><th>Created</th></tr></thead>
          <tbody>
            {recentRuns.length === 0 ? (
              <tr><td colSpan={5} style={{ textAlign: 'center', color: 'var(--text-muted)' }}>No runs yet</td></tr>
            ) : recentRuns.map(r => (
              <tr key={r.id}>
                <td style={{ fontFamily: 'monospace', fontSize: '0.8rem' }}>{r.id.slice(0, 8)}...</td>
                <td>{r.dag_id}</td>
                <td><StatusBadge status={r.status} /></td>
                <td>{r.triggered_by}</td>
                <td>{new Date(r.created_at).toLocaleString()}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </>
  );
}

function DAGsPage() {
  const { data: dags, loading, refetch } = useFetch(`${API}/dags`);
  const [showSubmit, setShowSubmit] = useState(false);
  const [yaml, setYaml] = useState('');

  const submitDag = async () => {
    try {
      await fetch(`${API}/dags`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ yaml }),
      });
      setShowSubmit(false);
      setYaml('');
      refetch();
    } catch (e) {
      alert('Failed to submit DAG: ' + e.message);
    }
  };

  const triggerDag = async (dagId) => {
    try {
      await fetch(`${API}/runs`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ dag_id: dagId, triggered_by: 'ui' }),
      });
      refetch();
    } catch (e) {
      alert('Failed to trigger: ' + e.message);
    }
  };

  if (loading) return <div className="loading"><div className="spinner" /></div>;

  return (
    <>
      <div className="page-header">
        <h1 className="page-title">DAGs</h1>
        <button className="btn btn-primary" onClick={() => setShowSubmit(!showSubmit)}>
          {showSubmit ? '✕ Cancel' : '+ Submit DAG'}
        </button>
      </div>
      {showSubmit && (
        <div className="card" style={{ marginBottom: 20 }}>
          <textarea value={yaml} onChange={e => setYaml(e.target.value)} rows={12}
            placeholder="Paste your YAML DAG definition here..."
            style={{
              width: '100%', background: 'var(--bg-primary)', color: 'var(--text-primary)',
              border: '1px solid var(--border)', borderRadius: 8, padding: 16,
              fontFamily: 'monospace', fontSize: '0.85rem', resize: 'vertical',
            }} />
          <button className="btn btn-primary" style={{ marginTop: 12 }} onClick={submitDag}>Submit</button>
        </div>
      )}
      <div className="table-container">
        <table>
          <thead><tr><th>DAG ID</th><th>Name</th><th>Schedule</th><th>Active</th><th>Actions</th></tr></thead>
          <tbody>
            {(!dags || dags.length === 0) ? (
              <tr><td colSpan={5} className="empty-state">No DAGs registered</td></tr>
            ) : dags.map(d => (
              <tr key={d.id}>
                <td style={{ fontWeight: 600 }}>{d.dag_id}</td>
                <td>{d.name}</td>
                <td style={{ fontFamily: 'monospace', fontSize: '0.8rem' }}>{d.schedule || '—'}</td>
                <td>{d.is_active ? '✅' : '❌'}</td>
                <td>
                  <button className="btn btn-outline btn-sm" onClick={() => triggerDag(d.dag_id)}>▶ Trigger</button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </>
  );
}

function RunsPage() {
  const { data: runs, loading } = useFetch(`${API}/runs`, 3000);
  const [selectedRun, setSelectedRun] = useState(null);
  const { data: tasks } = useFetch(selectedRun ? `${API}/runs/${selectedRun}/tasks` : null, 3000);

  if (loading) return <div className="loading"><div className="spinner" /></div>;

  return (
    <>
      <div className="page-header"><h1 className="page-title">Runs</h1></div>
      <div style={{ display: 'flex', gap: 20 }}>
        <div style={{ flex: 1 }}>
          <div className="table-container">
            <table>
              <thead><tr><th>Run ID</th><th>DAG</th><th>Status</th><th>Created</th></tr></thead>
              <tbody>
                {(!runs || runs.length === 0) ? (
                  <tr><td colSpan={4} style={{ textAlign: 'center', color: 'var(--text-muted)' }}>No runs</td></tr>
                ) : runs.map(r => (
                  <tr key={r.id} onClick={() => setSelectedRun(r.id)}
                    style={{ cursor: 'pointer', background: selectedRun === r.id ? 'var(--accent-glow)' : undefined }}>
                    <td style={{ fontFamily: 'monospace', fontSize: '0.8rem' }}>{r.id.slice(0, 8)}...</td>
                    <td>{r.dag_id}</td>
                    <td><StatusBadge status={r.status} /></td>
                    <td>{new Date(r.created_at).toLocaleString()}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
        {selectedRun && tasks && (
          <div style={{ flex: 1 }}>
            <h2 style={{ fontSize: '1rem', marginBottom: 12, color: 'var(--text-secondary)' }}>
              Tasks for {selectedRun.slice(0, 8)}...
            </h2>
            <DagGraph tasks={tasks} />
            <div className="table-container" style={{ marginTop: 16 }}>
              <table>
                <thead><tr><th>Task</th><th>Status</th><th>Attempt</th><th>Worker</th></tr></thead>
                <tbody>
                  {tasks.map(t => (
                    <tr key={t.id}>
                      <td style={{ fontWeight: 500 }}>{t.task_id}</td>
                      <td><StatusBadge status={t.status} /></td>
                      <td>{t.attempt}/{t.max_retries}</td>
                      <td style={{ fontFamily: 'monospace', fontSize: '0.8rem' }}>{t.worker_id || '—'}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )}
      </div>
    </>
  );
}

function DagGraph({ tasks }) {
  if (!tasks || tasks.length === 0) return null;
  return (
    <div className="dag-graph">
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: 12, justifyContent: 'center' }}>
        {tasks.map(t => (
          <div key={t.id} className={`dag-node ${t.status}`}>
            <span>{t.status === 'success' ? '✅' : t.status === 'failed' ? '❌' : t.status === 'running' ? '🔄' : '⏳'}</span>
            <span>{t.task_id}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

function WorkersPage() {
  const { data: workers, loading } = useFetch(`${API}/workers`);
  if (loading) return <div className="loading"><div className="spinner" /></div>;
  return (
    <>
      <div className="page-header"><h1 className="page-title">Workers</h1></div>
      <div className="table-container">
        <table>
          <thead><tr><th>Worker ID</th><th>Last Heartbeat</th><th>Status</th></tr></thead>
          <tbody>
            {(!workers || workers.length === 0) ? (
              <tr><td colSpan={3} style={{ textAlign: 'center', color: 'var(--text-muted)' }}>No active workers</td></tr>
            ) : workers.map(w => (
              <tr key={w.worker_id}>
                <td style={{ fontFamily: 'monospace' }}>{w.worker_id}</td>
                <td>{new Date(w.timestamp).toLocaleString()}</td>
                <td><StatusBadge status="running" /></td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </>
  );
}

export default function App() {
  const [page, setPage] = useState('dashboard');
  const pages = { dashboard: Dashboard, dags: DAGsPage, runs: RunsPage, workers: WorkersPage };
  const Page = pages[page] || Dashboard;
  return (
    <div className="app">
      <Sidebar page={page} setPage={setPage} />
      <main className="main-content"><Page /></main>
    </div>
  );
}
