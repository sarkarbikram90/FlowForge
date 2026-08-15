import React, { useState } from 'react';
import { ShieldCheck, Search } from 'lucide-react';
import { AuditLog } from '../types';

interface AuditPageProps {
  logs: AuditLog[];
}

export const AuditPage: React.FC<AuditPageProps> = ({ logs }) => {
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedLog, setSelectedLog] = useState<AuditLog | null>(null);

  const filtered = logs.filter(
    (l) =>
      l.action.toLowerCase().includes(searchTerm.toLowerCase()) ||
      l.actor.toLowerCase().includes(searchTerm.toLowerCase()) ||
      l.resource_type.toLowerCase().includes(searchTerm.toLowerCase())
  );

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-extrabold tracking-tight text-white flex items-center space-x-2">
            <ShieldCheck className="w-5 h-5 text-cyan-400" />
            <span>Immutable Audit Trail</span>
          </h1>
          <p className="text-xs text-slate-400 mt-1">
            Cryptographically sealed and immutable access logs for compliance and security auditing.
          </p>
        </div>

        <div className="relative">
          <Search className="w-3.5 h-3.5 text-slate-500 absolute left-3 top-1/2 transform -translate-y-1/2" />
          <input
            type="text"
            placeholder="Search audit trail..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="bg-slate-900 border border-slate-800 rounded-lg pl-9 pr-3 py-1.5 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-cyan-500 w-56 font-mono"
          />
        </div>
      </div>

      {/* Audit Log Table */}
      <div className="rounded-2xl bg-slate-900/70 border border-slate-800 overflow-hidden">
        <table className="w-full text-left text-xs font-mono">
          <thead className="bg-slate-950/80 border-b border-slate-800 text-slate-400 font-bold uppercase tracking-wider text-[11px]">
            <tr>
              <th className="px-5 py-3">Timestamp</th>
              <th className="px-5 py-3">Actor</th>
              <th className="px-5 py-3">Action</th>
              <th className="px-5 py-3">Resource Type</th>
              <th className="px-5 py-3">IP Address</th>
              <th className="px-5 py-3 text-right">Outcome</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-800/60 text-slate-300">
            {filtered.map((log) => (
              <tr
                key={log.id}
                onClick={() => setSelectedLog(log)}
                className="hover:bg-slate-800/40 transition-colors cursor-pointer"
              >
                <td className="px-5 py-3.5 text-slate-400">
                  {new Date(log.timestamp).toLocaleString()}
                </td>
                <td className="px-5 py-3.5 font-semibold text-white">{log.actor}</td>
                <td className="px-5 py-3.5">
                  <span className="px-2 py-0.5 rounded bg-slate-800 text-cyan-300 border border-slate-700 font-bold">
                    {log.action}
                  </span>
                </td>
                <td className="px-5 py-3.5 text-slate-300">{log.resource_type}</td>
                <td className="px-5 py-3.5 text-slate-400">{log.ip_address || '10.0.4.12'}</td>
                <td className="px-5 py-3.5 text-right">
                  <span className="px-2 py-0.5 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 font-bold text-[10px]">
                    {log.result}
                  </span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* JSON Payload Inspector Drawer */}
      {selectedLog && (
        <div className="p-5 rounded-2xl bg-slate-950 border border-slate-800 space-y-3 font-mono">
          <div className="flex items-center justify-between border-b border-slate-800 pb-2">
            <span className="text-xs font-bold text-cyan-400">Audit Record Metadata: {selectedLog.id}</span>
            <button
              onClick={() => setSelectedLog(null)}
              className="text-xs text-slate-500 hover:text-slate-300"
            >
              Close
            </button>
          </div>
          <pre className="text-xs text-slate-300 overflow-x-auto p-3 bg-slate-900/60 rounded-xl">
            {JSON.stringify(selectedLog, null, 2)}
          </pre>
        </div>
      )}
    </div>
  );
};
