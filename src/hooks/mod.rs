use crate::config::Config;
use crate::notes::{NotesFile, YamlMeta};
use crate::{Annotation, Task};
use log::debug;
use std::path::{Path, PathBuf};

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

    fn note_file_path(&self, task: &Task) -> PathBuf {
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

    fn create_path_annotation(&self, task: &mut Task, path: &Path) {
        let path_annotation = Annotation::new(path.to_str().unwrap_or("<invalid path>"));
        task.annotations.push(path_annotation);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::tempdir;

    pub fn test_cfg(dir: &Path) -> &'static Config {
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
}
