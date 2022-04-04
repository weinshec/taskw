use crate::config::Config;
use crate::notes::{NotesFile, YamlMeta};
use crate::{Annotation, Task};
use log::debug;
use std::path::PathBuf;

mod on_add;
mod on_modify;

pub type Feedback = String;

pub struct Hooks {
    config: &'static Config,
}

impl Hooks {
    pub fn with_config(cfg: &'static Config) -> Self {
        Self { config: cfg }
    }

    pub fn note_file_path(&self, task: &Task) -> PathBuf {
        PathBuf::from(&self.config.notes_dir)
            .join(task.uuid.to_string())
            .with_extension(&self.config.notes_ext)
    }

    fn create_notes_file(&self, task: &Task) -> Result<PathBuf, &'static str> {
        let path = self.note_file_path(task);

        let notes_file = NotesFile::new(&path)
            .with_header(YamlMeta::new(
                &task.description,
                task.entry.naive_local().date(),
            ))
            .with_content("%% Add your notes here");

        debug!("Creating note at {:?}", path);

        notes_file.write()?;
        Ok(path)
    }

    fn remove_notes_file(&self, task: &Task) -> Result<(), &'static str> {
        let path = self.note_file_path(task);
        std::fs::remove_file(&path).map_err(|_| "Cannot remove notes file")
    }

    fn create_path_annotation(&self, task: &mut Task) {
        let path = self.note_file_path(task);
        let path_annotation = Annotation::new(&format!(
            "taskw:note {}",
            path.to_str().unwrap_or("<invalid path>")
        ));
        task.annotations.push(path_annotation);
    }

    fn remove_path_annotation(&self, task: &mut Task) {
        task.annotations
            .retain(|annotation| !annotation.description.starts_with("taskw:note"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{tempdir, TempDir};

    pub fn test_config() -> (&'static Config, TempDir) {
        let temp_dir = tempdir().expect("tempdir creation succeeds");
        let mut cfg = Config::default();
        cfg.notes_dir = temp_dir.path().to_path_buf();
        (cfg.to_static(), temp_dir)
    }

    #[test]
    fn create_note_file_path_for_task() {
        let (cfg, _tmp_dir) = test_config();
        let task = Task::new("Dummy Task");
        let path = Hooks::with_config(&cfg).note_file_path(&task);

        let path_str = path.to_str().expect("valid path");
        assert!(path_str.contains(cfg.notes_dir.to_str().expect("valid path")));
        assert!(path_str.contains(&task.uuid.to_string()));
        assert!(path_str.contains(&cfg.notes_ext));
    }

    #[test]
    fn create_and_remove_notes_file() {
        let (cfg, _tmp_dir) = test_config();
        let hooks = Hooks::with_config(&cfg);
        let task = Task::new("Dummy Task");

        let path = hooks
            .create_notes_file(&task)
            .expect("file creation succeeds");
        assert!(path.exists());

        hooks
            .remove_notes_file(&task)
            .expect("file removal succeeds");
        assert!(!path.exists());
    }

    #[test]
    fn create_and_remove_path_annotation() {
        let (cfg, tmp_dir) = test_config();
        let hooks = Hooks::with_config(&cfg);
        let mut task = Task::new("Dummy Task");

        assert_eq!(task.annotations.len(), 0);
        hooks.create_path_annotation(&mut task);
        assert_eq!(task.annotations.len(), 1);

        let path_in_annotation = PathBuf::from(&task.annotations[0].description)
            .to_str()
            .expect("valid path")
            .to_owned();
        assert!(path_in_annotation.contains(tmp_dir.path().to_str().expect("valid path")));
        assert!(path_in_annotation.contains(&task.uuid.to_string()));

        assert_eq!(task.annotations.len(), 1);
        hooks.remove_path_annotation(&mut task);
        assert_eq!(task.annotations.len(), 0);
    }
}
