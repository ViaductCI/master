use std::fmt;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct Pipeline {
    pub name: String,
    pub stages: Vec<Stage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Stage {
    pub name: String,
    pub jobs: Vec<Job>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub name: String,
    pub repository: String,
    pub branch: String,
    pub commands: Vec<String>,
    #[serde(default)]
    pub inputs: Vec<JobInput>,
    #[serde(default)]
    pub outputs: Vec<JobOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobInput {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobOutput {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PipelineRun {
    pub id: String,
    pub pipeline_name: String,
    pub repository: String,
    pub branch: String,
    pub status: PipelineStatus,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_seconds: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PipelineStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl fmt::Display for PipelineStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PipelineStatus::Pending => write!(f, "pending"),
            PipelineStatus::Running => write!(f, "running"),
            PipelineStatus::Completed => write!(f, "completed"),
            PipelineStatus::Failed => write!(f, "failed"),
            PipelineStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}