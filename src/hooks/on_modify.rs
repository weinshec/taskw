use crate::Task;
use log::debug;

use super::{Feedback, Hooks};

impl Hooks {
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
}

#[cfg(test)]
mod tests {
    use super::{super::tests::test_cfg, *};
    use crate::Task;
    use std::path::PathBuf;
    use tempfile::tempdir;

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
