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

        match (original.has_tag(notes_tag), modified.has_tag(notes_tag)) {
            // notes tag added
            (false, true) => {
                let path = self.create_notes_file(&modified)?;
                self.create_path_annotation(&mut modified);
                Ok((
                    modified,
                    format!("Created notes file at {}", path.display()),
                ))
            }

            // notes tag removed
            (true, false) => {
                self.remove_path_annotation(&mut modified);
                let feedback = match self.remove_notes_file(&modified) {
                    Ok(_) => String::from("Removed notes file"),
                    _ => String::from("No notes found"),
                };
                Ok((modified, feedback))
            }
            _ => Ok((modified, String::new())),
        }
        // TODO: take status::Deleted into account
    }
}

#[cfg(test)]
mod tests {
    use super::{super::tests::test_config, *};
    use crate::Task;

    #[test]
    fn add_and_remove_note_tag() {
        let (cfg, _tmp_dir) = test_config();
        let hooks = Hooks::with_config(cfg);

        let old_task = Task::new("Dummy Task");
        let new_task = old_task.clone().with_tag(&cfg.notes_tag);
        let (task_with, feedback) = hooks.on_modify(old_task, new_task).expect("succeeds");

        assert_eq!(task_with.annotations.len(), 1);
        assert!(hooks.note_file_path(&task_with).exists());
        assert!(feedback.contains("notes"));

        let mut task_without = task_with.clone();
        task_without.tags.remove(&cfg.notes_tag);
        let (task_final, feedback) = hooks.on_modify(task_with, task_without).expect("succeeds");

        assert_eq!(task_final.annotations.len(), 0);
        assert!(!hooks.note_file_path(&task_final).exists());
        assert!(feedback.contains("notes"));
    }
}
