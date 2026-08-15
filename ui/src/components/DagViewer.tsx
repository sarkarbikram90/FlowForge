import React, { useState } from 'react';
import { CheckCircle2, Clock, PlayCircle, AlertCircle, XCircle, Flame } from 'lucide-react';
import { TaskRun, TaskState } from '../types';

export interface DagNode {
  id: string;
  name: string;
  type: string;
  dependsOn: string[];
  status?: TaskState;
  duration_ms?: number;
}

interface DagViewerProps {
  nodes: DagNode[];
  onSelectNode?: (nodeId: string) => void;
  selectedNodeId?: string;
  highlightCriticalPath?: boolean;
}

export const DagViewer: React.FC<DagViewerProps> = ({
  nodes,
  onSelectNode,
  selectedNodeId,
  highlightCriticalPath = true,
}) => {
  const [zoom, setZoom] = useState(1);

  // Compute layered positions (roots at left, dependents right)
  const levels: Record<string, number> = {};

  // Assign level 0 to roots
  nodes.forEach((n) => {
    if (n.dependsOn.length === 0) {
      levels[n.id] = 0;
    }
  });

  // Assign levels to children
  let changed = true;
  let iterations = 0;
  while (changed && iterations < 10) {
    changed = false;
    iterations++;
    nodes.forEach((n) => {
      if (n.dependsOn.length > 0) {
        const maxDepLevel = Math.max(
          ...n.dependsOn.map((d) => (levels[d] !== undefined ? levels[d] : 0))
        );
        const newLevel = maxDepLevel + 1;
        if (levels[n.id] !== newLevel) {
          levels[n.id] = newLevel;
          changed = true;
        }
      }
    });
  }

  // Group nodes by level
  const levelGroups: Record<number, DagNode[]> = {};
  nodes.forEach((n) => {
    const lvl = levels[n.id] || 0;
    if (!levelGroups[lvl]) levelGroups[lvl] = [];
    levelGroups[lvl].push(n);
  });

  // Calculate layout coordinates
  const nodePositions: Record<string, { x: number; y: number }> = {};
  const COLUMN_WIDTH = 240;
  const ROW_HEIGHT = 100;

  Object.entries(levelGroups).forEach(([levelStr, groupNodes]) => {
    const level = parseInt(levelStr, 10);
    const startY = 80;
    groupNodes.forEach((node, idx) => {
      nodePositions[node.id] = {
        x: 60 + level * COLUMN_WIDTH,
        y: startY + idx * ROW_HEIGHT,
      };
    });
  });

  const getStatusIcon = (status?: TaskState) => {
    switch (status) {
      case 'SUCCEEDED':
        return <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />;
      case 'RUNNING':
        return <PlayCircle className="w-4 h-4 text-amber-400 shrink-0 animate-pulse" />;
      case 'FAILED':
      case 'DEAD_LETTER':
        return <XCircle className="w-4 h-4 text-rose-400 shrink-0" />;
      case 'READY':
      case 'DISPATCHED':
        return <Clock className="w-4 h-4 text-cyan-400 shrink-0" />;
      default:
        return <Clock className="w-4 h-4 text-slate-500 shrink-0" />;
    }
  };

  const getStatusBorder = (status?: TaskState, isSelected?: boolean) => {
    if (isSelected) return 'border-cyan-400 shadow-md shadow-cyan-500/20';
    switch (status) {
      case 'SUCCEEDED':
        return 'border-emerald-500/40 hover:border-emerald-500';
      case 'RUNNING':
        return 'border-amber-500/60 hover:border-amber-500 shadow-sm shadow-amber-500/20';
      case 'FAILED':
      case 'DEAD_LETTER':
        return 'border-rose-500/60 hover:border-rose-500 shadow-sm shadow-rose-500/20';
      default:
        return 'border-slate-800 hover:border-slate-700';
    }
  };

  return (
    <div className="relative w-full h-[450px] bg-[#070A10] rounded-2xl border border-slate-800 overflow-hidden select-none">
      {/* Canvas Controls */}
      <div className="absolute top-4 right-4 z-10 flex items-center space-x-2 bg-slate-900/90 backdrop-blur-md p-1.5 rounded-xl border border-slate-800">
        <button
          onClick={() => setZoom((z) => Math.max(0.6, z - 0.1))}
          className="w-7 h-7 flex items-center justify-center rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 text-xs font-bold font-mono"
        >
          -
        </button>
        <span className="text-[11px] font-mono text-slate-400 px-1">
          {Math.round(zoom * 100)}%
        </span>
        <button
          onClick={() => setZoom((z) => Math.min(1.5, z + 0.1))}
          className="w-7 h-7 flex items-center justify-center rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 text-xs font-bold font-mono"
        >
          +
        </button>
        <button
          onClick={() => setZoom(1)}
          className="px-2 py-1 text-[10px] font-mono rounded-lg text-slate-400 hover:text-white hover:bg-slate-800"
        >
          Reset
        </button>
      </div>

      {/* SVG Canvas with dependency lines */}
      <div
        className="w-full h-full transform origin-top-left transition-transform duration-100"
        style={{ transform: `scale(${zoom})` }}
      >
        <svg className="absolute inset-0 w-[2000px] h-[1000px] pointer-events-none">
          <defs>
            <marker
              id="arrow"
              viewBox="0 0 10 10"
              refX="6"
              refY="5"
              markerWidth="6"
              markerHeight="6"
              orient="auto-start-reverse"
            >
              <path d="M 0 1 L 8 5 L 0 9 z" fill="#334155" />
            </marker>
            <marker
              id="arrow-active"
              viewBox="0 0 10 10"
              refX="6"
              refY="5"
              markerWidth="6"
              markerHeight="6"
              orient="auto-start-reverse"
            >
              <path d="M 0 1 L 8 5 L 0 9 z" fill="#06B6D4" />
            </marker>
          </defs>

          {nodes.map((node) => {
            const targetPos = nodePositions[node.id];
            if (!targetPos) return null;

            return node.dependsOn.map((depId) => {
              const srcPos = nodePositions[depId];
              if (!srcPos) return null;

              const startX = srcPos.x + 180;
              const startY = srcPos.y + 30;
              const endX = targetPos.x;
              const endY = targetPos.y + 30;

              const c1X = startX + (endX - startX) / 2;
              const c1Y = startY;
              const c2X = startX + (endX - startX) / 2;
              const c2Y = endY;

              const isSucceeded = node.status === 'SUCCEEDED';

              return (
                <path
                  key={`${depId}->${node.id}`}
                  d={`M ${startX} ${startY} C ${c1X} ${c1Y}, ${c2X} ${c2Y}, ${endX} ${endY}`}
                  fill="none"
                  stroke={isSucceeded ? '#06B6D4' : '#1E293B'}
                  strokeWidth={isSucceeded ? 2 : 1.5}
                  markerEnd={isSucceeded ? 'url(#arrow-active)' : 'url(#arrow)'}
                />
              );
            });
          })}
        </svg>

        {/* Render DAG Nodes */}
        {nodes.map((node) => {
          const pos = nodePositions[node.id];
          if (!pos) return null;
          const isSelected = selectedNodeId === node.id;

          return (
            <div
              key={node.id}
              onClick={() => onSelectNode?.(node.id)}
              style={{ left: `${pos.x}px`, top: `${pos.y}px` }}
              className={`absolute w-[180px] p-3 rounded-xl bg-slate-900/90 backdrop-blur-md border cursor-pointer transition-all ${getStatusBorder(
                node.status,
                isSelected
              )}`}
            >
              <div className="flex items-center justify-between mb-1.5">
                <div className="flex items-center space-x-2">
                  {getStatusIcon(node.status)}
                  <span className="font-mono text-xs font-semibold text-white truncate max-w-[100px]">
                    {node.id}
                  </span>
                </div>
                <span className="text-[10px] font-mono uppercase px-1.5 py-0.5 rounded bg-slate-800 text-slate-400 border border-slate-700">
                  {node.type}
                </span>
              </div>

              <div className="flex items-center justify-between text-[11px] font-mono text-slate-400 pt-1 border-t border-slate-800/80">
                <span>{node.status || 'PENDING'}</span>
                {node.duration_ms !== undefined && (
                  <span className="text-cyan-400 font-semibold">{node.duration_ms}ms</span>
                )}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};
