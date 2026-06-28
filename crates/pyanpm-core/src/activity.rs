use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use directories::ProjectDirs;
use fs_err as fs;

use crate::error::{PyanpmError, Result};
use crate::operations::ActivityRecord;

#[derive(Debug, Clone)]
pub struct ActivityStore {
    path: PathBuf,
}

impl ActivityStore {
    pub fn new() -> Result<Self> {
        let project_dirs = ProjectDirs::from("dev", "bblocks", "pyanpm")
            .ok_or(PyanpmError::MissingDefaultConfigDir)?;
        let path = project_dirs.data_local_dir().join("activity.jsonl");
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        if !path.exists() {
            fs::write(&path, "")?;
        }
        Ok(Self { path })
    }

    pub fn append(&self, record: &ActivityRecord) -> Result<()> {
        let mut file = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&self.path)?;
        let line = serde_json::to_string(record)?;
        writeln!(file, "{line}")?;
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<ActivityRecord>> {
        self.read_all()
    }

    pub fn latest_for_plugin(&self, plugin_name: &str) -> Result<Option<ActivityRecord>> {
        Ok(self
            .read_all()?
            .into_iter()
            .rev()
            .find(|record| record.plugin_names.iter().any(|name| name == plugin_name)))
    }

    pub fn get(&self, id: &str) -> Result<ActivityRecord> {
        self.read_all()?
            .into_iter()
            .find(|record| record.id == id)
            .ok_or_else(|| PyanpmError::ActivityNotFound(id.to_owned()))
    }

    pub fn clear(&self) -> Result<usize> {
        let records = self.read_all()?;
        let total_before = records.len();
        let kept = Vec::new();
        self.write_all(&kept)?;
        Ok(total_before.saturating_sub(kept.len()))
    }

    fn read_all(&self) -> Result<Vec<ActivityRecord>> {
        let file = fs::OpenOptions::new().read(true).create(true).truncate(false).open(&self.path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            records.push(serde_json::from_str::<ActivityRecord>(&line)?);
        }
        Ok(records)
    }

    fn write_all(&self, records: &[ActivityRecord]) -> Result<()> {
        let mut file = fs::File::create(&self.path)?;
        for record in records {
            let line = serde_json::to_string(record)?;
            writeln!(file, "{line}")?;
        }
        Ok(())
    }
}
