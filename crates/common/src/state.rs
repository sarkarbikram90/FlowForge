use serde::{Deserialize, Serialize};
use std::fmt;
use crate::error::{FlowForgeError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkflowState {
    Pending,
    Queued,
    Running,
    Paused,
    Succeeded,
    Failed,
    Canceling,
    Canceled,
    TimedOut,
    Retrying,
    Suspended,
}

impl WorkflowState {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            WorkflowState::Succeeded
                | WorkflowState::Failed
                | WorkflowState::Canceled
                | WorkflowState::TimedOut
        )
    }

    pub fn is_active(&self) -> bool {
        matches!(
            self,
            WorkflowState::Pending
                | WorkflowState::Queued
                | WorkflowState::Running
                | WorkflowState::Paused
                | WorkflowState::Canceling
                | WorkflowState::Retrying
                | WorkflowState::Suspended
        )
    }

    pub fn can_transition_to(&self, next: WorkflowState) -> bool {
        if *self == next {
            return true;
        }
        match self {
            WorkflowState::Pending => matches!(
                next,
                WorkflowState::Queued | WorkflowState::Running | WorkflowState::Canceled
            ),
            WorkflowState::Queued => matches!(
                next,
                WorkflowState::Running | WorkflowState::Canceled | WorkflowState::Failed
            ),
            WorkflowState::Running => matches!(
                next,
                WorkflowState::Succeeded
                    | WorkflowState::Failed
                    | WorkflowState::Canceling
                    | WorkflowState::Paused
                    | WorkflowState::TimedOut
                    | WorkflowState::Retrying
                    | WorkflowState::Suspended
            ),
            WorkflowState::Paused => matches!(
                next,
                WorkflowState::Running | WorkflowState::Canceling | WorkflowState::Canceled
            ),
            WorkflowState::Canceling => matches!(
                next,
                WorkflowState::Canceled | WorkflowState::Failed
            ),
            WorkflowState::Retrying => matches!(
                next,
                WorkflowState::Running | WorkflowState::Canceling | WorkflowState::Failed
            ),
            WorkflowState::Suspended => matches!(
                next,
                WorkflowState::Running | WorkflowState::Canceling | WorkflowState::Failed
            ),
            WorkflowState::Succeeded
            | WorkflowState::Failed
            | WorkflowState::Canceled
            | WorkflowState::TimedOut => false, // Terminal states cannot transition
        }
    }

    pub fn transition(&self, next: WorkflowState, run_id: &str) -> Result<WorkflowState> {
        if self.can_transition_to(next) {
            Ok(next)
        } else {
            Err(FlowForgeError::InvalidStateTransition {
                entity_type: "WorkflowRun".to_string(),
                id: run_id.to_string(),
                from: self.to_string(),
                to: next.to_string(),
            })
        }
    }
}

impl fmt::Display for WorkflowState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            WorkflowState::Pending => "PENDING",
            WorkflowState::Queued => "QUEUED",
            WorkflowState::Running => "RUNNING",
            WorkflowState::Paused => "PAUSED",
            WorkflowState::Succeeded => "SUCCEEDED",
            WorkflowState::Failed => "FAILED",
            WorkflowState::Canceling => "CANCELING",
            WorkflowState::Canceled => "CANCELED",
            WorkflowState::TimedOut => "TIMED_OUT",
            WorkflowState::Retrying => "RETRYING",
            WorkflowState::Suspended => "SUSPENDED",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskState {
    Pending,
    Blocked,
    Ready,
    Dispatched,
    Running,
    Succeeded,
    Failed,
    RetryWait,
    Canceled,
    TimedOut,
    Lost,
    DeadLetter,
}

impl TaskState {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            TaskState::Succeeded
                | TaskState::Failed
                | TaskState::Canceled
                | TaskState::TimedOut
                | TaskState::DeadLetter
        )
    }

    pub fn is_running(&self) -> bool {
        matches!(self, TaskState::Running)
    }

    pub fn can_transition_to(&self, next: TaskState) -> bool {
        if *self == next {
            return true;
        }
        match self {
            TaskState::Pending => matches!(
                next,
                TaskState::Blocked | TaskState::Ready | TaskState::Canceled
            ),
            TaskState::Blocked => matches!(
                next,
                TaskState::Ready | TaskState::Canceled | TaskState::Failed
            ),
            TaskState::Ready => matches!(
                next,
                TaskState::Dispatched | TaskState::Canceled
            ),
            TaskState::Dispatched => matches!(
                next,
                TaskState::Running | TaskState::Lost | TaskState::Canceled | TaskState::RetryWait
            ),
            TaskState::Running => matches!(
                next,
                TaskState::Succeeded
                    | TaskState::Failed
                    | TaskState::TimedOut
                    | TaskState::Lost
                    | TaskState::Canceled
                    | TaskState::RetryWait
            ),
            TaskState::RetryWait => matches!(
                next,
                TaskState::Ready | TaskState::DeadLetter | TaskState::Canceled
            ),
            TaskState::Lost => matches!(
                next,
                TaskState::RetryWait | TaskState::DeadLetter | TaskState::Failed | TaskState::Ready
            ),
            TaskState::Succeeded
            | TaskState::Failed
            | TaskState::Canceled
            | TaskState::TimedOut
            | TaskState::DeadLetter => false, // Terminal states cannot transition
        }
    }

    pub fn transition(&self, next: TaskState, task_run_id: &str) -> Result<TaskState> {
        if self.can_transition_to(next) {
            Ok(next)
        } else {
            Err(FlowForgeError::InvalidStateTransition {
                entity_type: "TaskRun".to_string(),
                id: task_run_id.to_string(),
                from: self.to_string(),
                to: next.to_string(),
            })
        }
    }
}

impl fmt::Display for TaskState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TaskState::Pending => "PENDING",
            TaskState::Blocked => "BLOCKED",
            TaskState::Ready => "READY",
            TaskState::Dispatched => "DISPATCHED",
            TaskState::Running => "RUNNING",
            TaskState::Succeeded => "SUCCEEDED",
            TaskState::Failed => "FAILED",
            TaskState::RetryWait => "RETRY_WAIT",
            TaskState::Canceled => "CANCELED",
            TaskState::TimedOut => "TIMED_OUT",
            TaskState::Lost => "LOST",
            TaskState::DeadLetter => "DEAD_LETTER",
        };
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_workflow_transitions() {
        let state = WorkflowState::Pending;
        assert!(state.can_transition_to(WorkflowState::Queued));
        let state = WorkflowState::Queued;
        assert!(state.can_transition_to(WorkflowState::Running));
        let state = WorkflowState::Running;
        assert!(state.can_transition_to(WorkflowState::Succeeded));
    }

    #[test]
    fn test_terminal_workflow_cannot_transition() {
        let state = WorkflowState::Succeeded;
        assert!(!state.can_transition_to(WorkflowState::Running));
        assert!(state.transition(WorkflowState::Running, "run-1").is_err());
    }

    #[test]
    fn test_task_state_transitions() {
        let state = TaskState::Pending;
        assert!(state.can_transition_to(TaskState::Ready));
        let state = TaskState::Ready;
        assert!(state.can_transition_to(TaskState::Dispatched));
        let state = TaskState::Dispatched;
        assert!(state.can_transition_to(TaskState::Running));
        let state = TaskState::Running;
        assert!(state.can_transition_to(TaskState::Lost));
        let state = TaskState::Lost;
        assert!(state.can_transition_to(TaskState::RetryWait));
        let state = TaskState::RetryWait;
        assert!(state.can_transition_to(TaskState::Ready));
    }
}
