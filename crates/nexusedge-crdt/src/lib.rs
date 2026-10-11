//! nexusedge-crdt: Delta-State CRDT task graph and result receipt log.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskState {
    Submitted,
    InQueue,
    Executing { node_id: String },
    Completed { result_hash: String },
    Failed { error: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRecord {
    pub task_id: String,
    pub payload_hash: String,
    pub state: TaskState,
    pub timestamp: u64,
    pub signature: String,
}

pub struct CrdtLog {
    records: Vec<TaskRecord>,
}

impl CrdtLog {
    pub fn new() -> Self {
        Self { records: Vec::new() }
    }

    pub fn append(&mut self, record: TaskRecord) {
        self.records.push(record);
    }

    pub fn records(&self) -> &[TaskRecord] {
        &self.records
    }
}

impl Default for CrdtLog {
    fn default() -> Self {
        Self::new()
    }
}
