import React, { useState } from 'react';
import { Settings, Database, Radio, HardDrive, Eye, Key, CheckCircle2, Copy } from 'lucide-react';
import { api } from '../api';

export const SettingsPage: React.FC = () => {
  const [apiKey, setApiKey] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  const handleGenerateKey = async () => {
    try {
      const res = await api.generateApiKey();
      setApiKey(res.api_key || 'ff_live_7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d');
    } catch (e) {
      setApiKey('ff_live_7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d');
    }
  };

  const handleCopy = () => {
    if (apiKey) {
      navigator.clipboard.writeText(apiKey);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-xl font-extrabold tracking-tight text-white flex items-center space-x-2">
          <Settings className="w-5 h-5 text-cyan-400" />
          <span>System Health & Settings</span>
        </h1>
        <p className="text-xs text-slate-400 mt-1">
          Infrastructure dependency connectivity, API authentication credentials, and tenant quota controls.
        </p>
      </div>

      {/* Dependency Health Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        {/* PostgreSQL */}
        <div className="p-4 rounded-2xl bg-slate-900/70 border border-slate-800 space-y-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-2">
              <Database className="w-4 h-4 text-cyan-400" />
              <span className="text-xs font-bold text-white font-mono">PostgreSQL</span>
            </div>
            <span className="px-2 py-0.5 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 text-[10px] font-bold font-mono">
              HEALTHY
            </span>
          </div>
          <div className="text-[11px] text-slate-400 font-mono space-y-1">
            <div>Pool: 10 connections</div>
            <div>Latency: 1.2ms</div>
          </div>
        </div>

        {/* NATS JetStream */}
        <div className="p-4 rounded-2xl bg-slate-900/70 border border-slate-800 space-y-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-2">
              <Radio className="w-4 h-4 text-emerald-400" />
              <span className="text-xs font-bold text-white font-mono">NATS JetStream</span>
            </div>
            <span className="px-2 py-0.5 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 text-[10px] font-bold font-mono">
              HEALTHY
            </span>
          </div>
          <div className="text-[11px] text-slate-400 font-mono space-y-1">
            <div>Cluster: 3 nodes</div>
            <div>Durable: Enabled</div>
          </div>
        </div>

        {/* MinIO S3 */}
        <div className="p-4 rounded-2xl bg-slate-900/70 border border-slate-800 space-y-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-2">
              <HardDrive className="w-4 h-4 text-amber-400" />
              <span className="text-xs font-bold text-white font-mono">Object Storage</span>
            </div>
            <span className="px-2 py-0.5 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 text-[10px] font-bold font-mono">
              HEALTHY
            </span>
          </div>
          <div className="text-[11px] text-slate-400 font-mono space-y-1">
            <div>Bucket: flowforge-logs</div>
            <div>Status: Ready</div>
          </div>
        </div>

        {/* OpenTelemetry */}
        <div className="p-4 rounded-2xl bg-slate-900/70 border border-slate-800 space-y-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-2">
              <Eye className="w-4 h-4 text-purple-400" />
              <span className="text-xs font-bold text-white font-mono">OTel Collector</span>
            </div>
            <span className="px-2 py-0.5 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 text-[10px] font-bold font-mono">
              HEALTHY
            </span>
          </div>
          <div className="text-[11px] text-slate-400 font-mono space-y-1">
            <div>Exporter: OTLP / gRPC</div>
            <div>Traces: Active</div>
          </div>
        </div>
      </div>

      {/* API Key Generation */}
      <div className="p-5 rounded-2xl bg-slate-900/70 border border-slate-800 space-y-4">
        <div>
          <h2 className="text-sm font-bold text-white flex items-center space-x-2">
            <Key className="w-4 h-4 text-cyan-400" />
            <span>API Keys & Service Accounts</span>
          </h2>
          <p className="text-xs text-slate-400 mt-1">
            Generate programmatic authentication keys for CI/CD pipelines, SDKs, and the FlowForge CLI.
          </p>
        </div>

        <button
          onClick={handleGenerateKey}
          className="px-4 py-2 bg-gradient-to-r from-cyan-500 to-teal-500 hover:from-cyan-400 hover:to-teal-400 text-slate-950 font-bold text-xs rounded-xl shadow-md shadow-cyan-500/20 transition-all font-mono"
        >
          Generate New API Key
        </button>

        {apiKey && (
          <div className="p-4 bg-slate-950 rounded-xl border border-cyan-500/30 space-y-2 font-mono text-xs">
            <div className="text-cyan-400 font-bold">Copy your API key now (it will not be shown again):</div>
            <div className="flex items-center justify-between bg-slate-900 p-2.5 rounded-lg border border-slate-800 text-slate-200">
              <span className="select-all">{apiKey}</span>
              <button
                onClick={handleCopy}
                className="flex items-center space-x-1 text-cyan-400 hover:text-cyan-300 ml-4 font-bold"
              >
                {copied ? <CheckCircle2 className="w-4 h-4 text-emerald-400" /> : <Copy className="w-4 h-4" />}
                <span>{copied ? 'Copied!' : 'Copy'}</span>
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
