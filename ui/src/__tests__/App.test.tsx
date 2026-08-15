import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Navbar } from '../components/Navbar';
import { Sidebar } from '../components/Sidebar';
import { DagViewer } from '../components/DagViewer';

describe('FlowForge UI Components', () => {
  it('renders Navbar with title and quick actions', () => {
    render(
      <Navbar
        stats={{
          active_workflows: 5,
          total_runs: 100,
          running_runs: 2,
          succeeded_runs: 95,
          failed_runs: 3,
          queued_tasks: 4,
          running_tasks: 2,
          active_workers: 3,
          dlq_count: 0,
          scheduler_leader_id: 'sched-1',
          scheduler_healthy: true,
          success_rate: 97.0,
          average_duration_ms: 2500,
        }}
        onOpenApplyModal={() => {}}
        onOpenTriggerModal={() => {}}
      />
    );

    expect(screen.getByText('FlowForge')).toBeInTheDocument();
    expect(screen.getByText('Apply Workflow')).toBeInTheDocument();
    expect(screen.getByText('Trigger Run')).toBeInTheDocument();
  });

  it('renders Sidebar with operations tabs', () => {
    render(
      <Sidebar
        activeTab="overview"
        setActiveTab={() => {}}
        dlqCount={2}
      />
    );

    expect(screen.getByText('Overview')).toBeInTheDocument();
    expect(screen.getByText('Workflows')).toBeInTheDocument();
    expect(screen.getByText('Workers Fleet')).toBeInTheDocument();
    expect(screen.getByText('Queues & DLQ')).toBeInTheDocument();
    expect(screen.getByText('2')).toBeInTheDocument();
  });

  it('renders DagViewer with task nodes', () => {
    const sampleNodes = [
      { id: 'extract', name: 'Extract', type: 'shell', dependsOn: [] },
      { id: 'transform', name: 'Transform', type: 'container', dependsOn: ['extract'] },
    ];

    render(<DagViewer nodes={sampleNodes} />);
    expect(screen.getByText('extract')).toBeInTheDocument();
    expect(screen.getByText('transform')).toBeInTheDocument();
  });
});
