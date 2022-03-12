use std::path::PathBuf;

/// Configuration for the taskwiki executable
pub struct Config {
    /// The taskwarrior tag indicating this task is eligible for notes file creation
    pub notes_tag: String,
    /// Base directory where notes files are created
    pub notes_dir: PathBuf,
    /// File extension used for notes files
    pub notes_ext: String,
}

impl Config {
    /// Make this `Config` object a reference bound by static lifetime
    pub fn to_static(self) -> &'static Self {
        let static_box = Box::new(self);
        Box::leak(static_box)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            notes_tag: String::from("wiki"),
            notes_dir: PathBuf::from("/home/weinshec/scratch"),
            notes_ext: String::from("md"),
        }
    }
}
