use crate::datetime_format;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use uuid::Uuid;

/// A Taskwarrior task
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Task {
    /// The state of the task, one of ["pending", "deleted", "completed", "waiting", "recurring"].
    pub status: Status,

    /// A 32-hex-character lower case string using hyphen-separators
    pub uuid: Uuid,

    /// UTC datetime of creation time represented in ISO 8601 combined date and time.
    #[serde(with = "datetime_format")]
    pub entry: DateTime<Utc>,

    /// Task description field (mandatory for all tasks)
    /// It may not contain newline characters, but may contain other characters, properly escaped.
    pub description: String,

    /// A project is a single string.
    /// Note that projects receive special handling when a "." is used, it implies a hierarchy.
    pub project: Option<String>,

    /// An array of strings, where each string is a single word containing no spaces.
    #[serde(default)]
    pub tags: HashSet<String>,

    /// Annotations are strings with timestamps.
    /// Each annotation itself has an `entry` field and a `description` field.
    #[serde(default)]
    pub annotations: Vec<Annotation>,

    /// Internally generated datetime this task has been modified at.
    #[serde(with = "datetime_format")]
    pub modified: DateTime<Utc>,

    /// All other attributes not explicitly captured by any other given field.
    #[serde(flatten)]
    pub unknown_fields: HashMap<String, Value>,
}

impl Task {
    pub fn new(description: &str) -> Self {
        let now = Utc::now();
        Self {
            status: Status::Pending,
            uuid: Uuid::new_v4(),
            entry: now,
            description: description.to_string(),
            project: None,
            tags: HashSet::new(),
            annotations: vec![],
            modified: now,
            unknown_fields: HashMap::new(),
        }
    }

    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.insert(tag.to_string());
        self
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(&tag.to_string())
    }
}

impl FromStr for Task {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // serde_json::from_str(s).map_err(|_| "Cannot deserialize task")
        Ok(serde_json::from_str(s).unwrap())
    }
}

impl std::fmt::Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let task_str = serde_json::to_string(&self).map_err(|_| std::fmt::Error {})?;
        write!(f, "{}", task_str)
    }
}

/// Status field of a taskwarrior task
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// A pending task is a task that has not yet been completed or deleted.
    Pending,
    /// A deleted task is one that has been removed from the pending state.
    /// It MUST have an "end" field specified.
    Deleted,
    /// A completed task is one that has been removed from the pending state by completion.
    /// It MUST have an "end" field specified.
    Completed,
    /// A waiting task is ostensibly a pending task that has been hidden from typical view.
    /// It MUST have an "wait" field specified.
    Waiting,
    /// A recurring task is essentially a parent template task from which child tasks are cloned.
    Recurring,
}

/// Annotations to a taskwarrior are pairs of "entry" (datetime) and "description" (String)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Annotation {
    #[serde(with = "datetime_format")]
    pub entry: DateTime<Utc>,
    pub description: String,
}

impl Annotation {
    pub fn new(description: &str) -> Self {
        let now = Utc::now();
        Self {
            entry: now,
            description: description.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    const TASK_JSON: &str = r#"
        {
            "description": "Dummy Task",
            "entry": "20220110T171619Z",
            "modified": "20220111T074112Z",
            "project": "dummy",
            "status": "pending",
            "uuid": "dde3720b-003f-4776-8e15-61e5d90376af",
            "annotations": [
                {"entry": "20220111T074112Z", "description": "note:dp"}
            ],
            "tags": ["wiki"],
            "user_defined": "custom_field"
        }
        "#;

    #[test]
    fn deserialize_all_fields_from_json() {
        let task: Task = serde_json::from_str(TASK_JSON).expect("deserialization succeeded");

        assert_eq!(
            task.uuid,
            Uuid::parse_str("dde3720b-003f-4776-8e15-61e5d90376af").unwrap()
        );
        assert_eq!(task.description, "Dummy Task");
        assert_eq!(task.project, Some("dummy".to_string()));
        assert_eq!(task.status, Status::Pending);
        assert_eq!(task.entry, Utc.ymd(2022, 1, 10).and_hms(17, 16, 19));
        assert_eq!(task.modified, Utc.ymd(2022, 1, 11).and_hms(7, 41, 12));
        assert!(task.tags.contains(&String::from("wiki")));
        assert_eq!(
            task.annotations,
            vec![Annotation {
                entry: Utc.ymd(2022, 1, 11).and_hms(7, 41, 12),
                description: String::from("note:dp"),
            }]
        );
        assert_eq!(task.unknown_fields["user_defined"], "custom_field");
    }

    #[test]
    fn serializing_datetimes_follows_taskwarrior_format() {
        let task: Task = serde_json::from_str(TASK_JSON).expect("deserialization succeeded");
        let serialized = serde_json::to_string(&task).expect("serialization succeeded");
        assert!(serialized.contains("20220110T171619Z"));
    }

    #[test]
    fn serializing_uuid_follows_taskwarrior_format() {
        let task: Task = serde_json::from_str(TASK_JSON).expect("deserialization succeeded");
        let serialized = serde_json::to_string(&task).expect("serialization succeeded");
        assert!(serialized.contains("dde3720b-003f-4776-8e15-61e5d90376af"));
    }

    #[test]
    fn task_has_tag() {
        let task: Task = serde_json::from_str(TASK_JSON).expect("deserialization succeeded");
        assert!(task.has_tag("wiki"));
        assert!(!task.has_tag("foobar"));
    }
}
