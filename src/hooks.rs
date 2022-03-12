use crate::config::Config;
use crate::notes::{NotesFile, YamlMeta};
use crate::{Annotation, Task};
use log::debug;
use std::path::PathBuf;

pub struct Hooks {
    config: &'static Config,
}

pub type Feedback = String;

impl Hooks {
    pub fn with_config(cfg: &'static Config) -> Self {
        Self { config: cfg }
    }

    pub fn on_add(&self, mut task: Task) -> Result<(Task, Feedback), &'static str> {
        debug!("added = {:#?}", task);

        if !task.has_tag(&self.config.notes_tag) {
            return Ok((task, "".to_string()));
        }

        let path = self.note_file_path(&task);
        let path_str = path.to_str().unwrap_or("<invalid path>");

        let notes_file = NotesFile::new(&path)
            .with_header(YamlMeta::new(
                &task.description,
                task.entry.naive_local().date(),
            ))
            .with_content("%% Add your notes here");

        task.annotations
            .push(Annotation::new(&format!("taskwiki:path {}", path_str)));

        debug!("Creating note at {}", path_str);
        notes_file.write()?;

        Ok((task, "".to_string()))
    }

    pub fn on_modify(
        &self,
        original: Task,
        modified: Task,
    ) -> Result<(Task, Feedback), &'static str> {
        debug!("original = {:#?}", original);
        debug!("modified = {:#?}", modified);
        Ok((modified, "".to_string()))
    }

    fn note_file_path(&self, task: &Task) -> PathBuf {
        PathBuf::from(&self.config.notes_dir)
            .join(task.uuid.to_string())
            .with_extension(&self.config.notes_ext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::tempdir;

    fn test_cfg(dir: &Path) -> &'static Config {
        let mut cfg = Config::default();
        cfg.notes_dir = dir.to_path_buf();
        cfg.to_static()
    }

    #[test]
    fn create_note_file_path_for_task() {
        let temp_dir = tempdir().expect("tempdir creation succeeds");
        let cfg = test_cfg(temp_dir.path());

        let task = Task::new("Dummy Task");
        let path = Hooks::with_config(&cfg).note_file_path(&task);
        let path_str = path.to_str().expect("valid path");

        assert!(path_str.contains(cfg.notes_dir.to_str().expect("valid path")));
        assert!(path_str.contains(&task.uuid.to_string()));
        assert!(path_str.contains(&cfg.notes_ext));
    }

    #[test]
    fn on_add_annotates_task_with_path_to_notes_file() {
        let temp_dir = tempdir().expect("tempdir creation succeeds");
        let cfg = test_cfg(temp_dir.path());

        let task = Task::new("Dummy Task").with_tag("wiki");
        let (task, _) = Hooks::with_config(cfg).on_add(task).expect("succeeds");
        assert_eq!(task.annotations.len(), 1);
        assert!(task.annotations[0]
            .description
            .contains(cfg.notes_dir.to_str().expect("valid path")));
    }
}
