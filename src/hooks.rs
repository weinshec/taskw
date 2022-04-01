use crate::config::Config;
use crate::notes::{NotesFile, YamlMeta};
use crate::{Annotation, Task};
use log::debug;
use std::path::{Path, PathBuf};

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

        let path = self.create_notes_file(&task)?;
        self.create_path_annotation(&mut task, &path);

        Ok((task, format!("Created notes file at {}", path.display())))
    }

    pub fn on_modify(
        &self,
        original: Task,
        mut modified: Task,
    ) -> Result<(Task, Feedback), &'static str> {
        debug!("original = {:#?}", original);
        debug!("modified = {:#?}", modified);

        let notes_tag = &self.config.notes_tag;

        if !original.has_tag(notes_tag) && modified.has_tag(notes_tag) {
            let path = self.create_notes_file(&modified)?;
            self.create_path_annotation(&mut modified, &path);
            return Ok((
                modified,
                format!("Created notes file at {}", path.display()),
            ));
        }

        if original.has_tag(notes_tag) && !modified.has_tag(notes_tag) {
            // TODO: remove notes file and annotation
            // TODO: take status::Deleted into account
        }

        Ok((modified, String::new()))
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
    fn on_add_passes_irrelevant_task() {
        let temp_dir = tempdir().expect("tempdir creation succeeds");
        let cfg = test_cfg(temp_dir.path());

        let task = Task::new("Dummy Task").with_tag("not_the_wiki_tag");
        let (returned_task, feedback) = Hooks::with_config(cfg)
            .on_add(task.clone())
            .expect("succeeds");
        assert_eq!(task, returned_task);
        assert_eq!(feedback, String::new());
    }

    #[test]
    fn on_add_creates_notes_file_and_annotates_task_with_path() {
        let temp_dir = tempdir().expect("tempdir creation succeeds");
        let cfg = test_cfg(temp_dir.path());

        let task = Task::new("Dummy Task").with_tag(&cfg.notes_tag);
        let (task, _) = Hooks::with_config(cfg).on_add(task).expect("succeeds");

        assert_eq!(task.annotations.len(), 1);
        let notes_file_path = PathBuf::from(&task.annotations[0].description);
        dbg!(&notes_file_path);
        assert!(notes_file_path
            .to_str()
            .expect("valid path")
            .contains(cfg.notes_dir.to_str().expect("valid path")));
        assert!(notes_file_path.exists());
    }

    #[test]
    fn on_modify_creates_notes_file_and_annotates_task_with_path() {
        let temp_dir = tempdir().expect("tempdir creation succeeds");
        let cfg = test_cfg(temp_dir.path());

        let old_task = Task::new("Dummy Task");
        let new_task = old_task.clone().with_tag(&cfg.notes_tag);
        let (task, _) = Hooks::with_config(cfg)
            .on_modify(old_task, new_task)
            .expect("succeeds");

        assert_eq!(task.annotations.len(), 1);
        let notes_file_path = PathBuf::from(&task.annotations[0].description);
        dbg!(&notes_file_path);
        assert!(notes_file_path
            .to_str()
            .expect("valid path")
            .contains(cfg.notes_dir.to_str().expect("valid path")));
        assert!(notes_file_path.exists());
    }
}
