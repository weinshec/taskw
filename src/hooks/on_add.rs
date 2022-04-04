use crate::Task;
use log::debug;

use super::{Feedback, Hooks};

impl Hooks {
    pub fn on_add(&self, mut task: Task) -> Result<(Task, Feedback), &'static str> {
        debug!("added = {:#?}", task);

        if !task.has_tag(&self.config.notes_tag) {
            return Ok((task, Feedback::new()));
        }

        let path = self.create_notes_file(&task)?;
        self.create_path_annotation(&mut task);

        Ok((task, format!("Created notes file at {}", path.display())))
    }
}

#[cfg(test)]
mod tests {
    use super::{super::tests::test_config, *};
    use crate::Task;

    #[test]
    fn on_add_passes_irrelevant_task() {
        let (cfg, _tmp_dir) = test_config();

        let task = Task::new("Dummy Task").with_tag("not_the_wiki_tag");
        let (returned_task, feedback) = Hooks::with_config(cfg)
            .on_add(task.clone())
            .expect("succeeds");

        assert_eq!(task, returned_task);
        assert_eq!(feedback, String::new());
    }

    #[test]
    fn on_add_creates_notes_file_and_annotates_task_with_path() {
        let (cfg, _tmp_dir) = test_config();
        let hooks = Hooks::with_config(cfg);

        let task = Task::new("Dummy Task").with_tag(&cfg.notes_tag);
        let (task, feedback) = hooks.on_add(task).expect("succeeds");

        assert_eq!(task.annotations.len(), 1);
        assert!(hooks.note_file_path(&task).exists());
        assert!(feedback.contains("notes"));
    }
}
