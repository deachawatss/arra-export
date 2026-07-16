use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::oracle_client::RemoteExportJob;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Json,
    Csv,
    Markdown,
    Jsonl,
}

impl ExportFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Csv => "csv",
            Self::Markdown => "md",
            Self::Jsonl => "jsonl",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportRequest {
    pub collection: String,
    pub format: ExportFormat,
    #[serde(default)]
    pub include_graph: bool,
}

impl ExportRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.collection.trim().is_empty() {
            return Err("collection is required");
        }
        if self.collection.len() > 200 {
            return Err("collection must be 200 characters or fewer");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportJob {
    pub id: String,
    pub collection: String,
    pub format: ExportFormat,
    pub include_graph: bool,
    pub status: String,
    pub progress: u8,
    pub filename: Option<String>,
    pub created_at: Option<String>,
    pub download_url: String,
}

#[derive(Debug, Clone)]
pub(crate) struct StoredExportJob {
    pub view: ExportJob,
    pub remote_job_id: String,
    pub oracle_url: url::Url,
}

#[derive(Debug, Clone, Default)]
pub struct JobManager {
    jobs: Arc<Mutex<Vec<StoredExportJob>>>,
}

impl JobManager {
    pub fn record(
        &self,
        request: &ExportRequest,
        remote: RemoteExportJob,
        oracle_url: url::Url,
    ) -> ExportJob {
        let id = remote.job_id;
        let view = ExportJob {
            download_url: format!("/api/export/{id}/download"),
            id: id.clone(),
            collection: request.collection.clone(),
            format: request.format,
            include_graph: request.include_graph,
            status: remote.status.unwrap_or_else(|| "completed".to_owned()),
            progress: remote.progress.unwrap_or(100).min(100),
            filename: remote.filename,
            created_at: remote.created_at,
        };
        let stored = StoredExportJob {
            view: view.clone(),
            remote_job_id: id,
            oracle_url,
        };

        let mut jobs = self.jobs.lock().expect("export job history lock poisoned");
        jobs.retain(|job| job.view.id != stored.view.id);
        jobs.insert(0, stored);
        jobs.truncate(100);
        view
    }

    pub fn history(&self) -> Vec<ExportJob> {
        self.jobs
            .lock()
            .expect("export job history lock poisoned")
            .iter()
            .map(|job| job.view.clone())
            .collect()
    }

    pub(crate) fn get(&self, id: &str) -> Option<StoredExportJob> {
        self.jobs
            .lock()
            .expect("export job history lock poisoned")
            .iter()
            .find(|job| job.view.id == id)
            .cloned()
    }
}

#[derive(Debug)]
pub struct ExportArtifact {
    pub bytes: Vec<u8>,
    pub content_type: String,
    pub filename: String,
}
