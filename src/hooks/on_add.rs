use crate::Task;
use log::debug;

use super::{Feedback, Hooks};

impl Hooks {
    pub fn on_add(&self, mut task: Task) -> Result<(Task, Feedback), &'static str> {
        debug!("added = {:#?}", task);

        if !task.has_tag(&self.config.notes_tag) {
            return Ok((task, "".to_string()));
        }

        let path = self.create_notes_file(&task)?;
        self.create_path_annotation(&mut task, &path);

        Ok((task, format!("Created notes file at {}", path.display())))
    }
}

#[cfg(test)]
mod tests {
    use super::{super::tests::test_cfg, *};
    use crate::Task;
    use std::path::PathBuf;
    use tempfile::tempdir;

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
}
