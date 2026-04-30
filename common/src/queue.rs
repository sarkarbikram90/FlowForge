use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use tracing::{info, debug};
use crate::models::TaskMessage;
use crate::error::Result;

const TASK_QUEUE: &str = "flowforge:tasks";
const PROCESSING_QUEUE: &str = "flowforge:processing";
const RESULT_QUEUE: &str = "flowforge:results";

#[derive(Clone)]
pub struct TaskQueue {
    conn: MultiplexedConnection,
}

impl TaskQueue {
    pub async fn new(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| crate::error::FlowForgeError::Internal(format!("Redis client error: {e}")))?;
        let conn = client.get_multiplexed_async_connection().await?;
        info!("Connected to Redis");
        Ok(Self { conn })
    }

    /// Push a task onto the queue for workers to pick up.
    pub async fn enqueue_task(&self, msg: &TaskMessage) -> Result<()> {
        let payload = serde_json::to_string(msg)?;
        let mut conn = self.conn.clone();
        conn.lpush::<_, _, ()>(TASK_QUEUE, &payload).await?;
        debug!(task_id = %msg.task_id, "Task enqueued");
        Ok(())
    }

    /// Blocking dequeue — moves item from task queue to processing queue atomically.
    /// Returns None if timeout (5 seconds) expires with no work.
    pub async fn dequeue_task(&self, timeout_secs: f64) -> Result<Option<TaskMessage>> {
        let mut conn = self.conn.clone();
        let result: Option<String> = redis::cmd("BRPOPLPUSH")
            .arg(TASK_QUEUE)
            .arg(PROCESSING_QUEUE)
            .arg(timeout_secs)
            .query_async(&mut conn)
            .await?;

        match result {
            Some(payload) => {
                let msg: TaskMessage = serde_json::from_str(&payload)?;
                debug!(task_id = %msg.task_id, "Task dequeued");
                Ok(Some(msg))
            }
            None => Ok(None),
        }
    }

    /// Acknowledge task completion — remove from processing queue.
    pub async fn ack_task(&self, msg: &TaskMessage) -> Result<()> {
        let payload = serde_json::to_string(msg)?;
        let mut conn = self.conn.clone();
        conn.lrem::<_, _, ()>(PROCESSING_QUEUE, 1, &payload).await?;
        Ok(())
    }

    /// Publish task result for the scheduler to consume.
    pub async fn publish_result(&self, result: &crate::models::TaskResult) -> Result<()> {
        let payload = serde_json::to_string(result)?;
        let mut conn = self.conn.clone();
        conn.lpush::<_, _, ()>(RESULT_QUEUE, &payload).await?;
        debug!(task_id = %result.task_id, "Result published");
        Ok(())
    }

    /// Non-blocking fetch of a task result. Returns None if queue is empty.
    pub async fn poll_result(&self) -> Result<Option<crate::models::TaskResult>> {
        let mut conn = self.conn.clone();
        let result: Option<String> = conn.rpop(RESULT_QUEUE, None).await?;
        match result {
            Some(payload) => {
                let r: crate::models::TaskResult = serde_json::from_str(&payload)?;
                Ok(Some(r))
            }
            None => Ok(None),
        }
    }

    /// Get the current queue depth.
    pub async fn queue_depth(&self) -> Result<i64> {
        let mut conn = self.conn.clone();
        let len: i64 = conn.llen(TASK_QUEUE).await?;
        Ok(len)
    }
}
