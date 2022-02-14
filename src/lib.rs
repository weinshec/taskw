mod datetime_format;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Task {
    description: String,
    project: Option<String>,
    status: Status,
    uuid: String,
    tags: Vec<String>,
    annotations: Vec<Annotation>,

    #[serde(with = "datetime_format")]
    entry: DateTime<Utc>,

    #[serde(with = "datetime_format")]
    modified: DateTime<Utc>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Status {
    Pending,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Annotation {
    #[serde(with = "datetime_format")]
    entry: DateTime<Utc>,
    description: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn deserialize_all_fields_from_json() {
        let task_json = r#"
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
                "tags": ["wiki"]
            }
        "#;

        let task: Task = serde_json::from_str(task_json).expect("parsing succeeded");

        assert_eq!(task.description, "Dummy Task");
        assert_eq!(task.project, Some("dummy".to_string()));
        assert_eq!(task.status, Status::Pending);
        assert_eq!(task.uuid, "dde3720b-003f-4776-8e15-61e5d90376af");
        assert_eq!(task.entry, Utc.ymd(2022, 1, 10).and_hms(17, 16, 19));
        assert_eq!(task.modified, Utc.ymd(2022, 1, 11).and_hms(7, 41, 12));
        assert_eq!(task.tags, vec![String::from("wiki")]);
        assert_eq!(
            task.annotations,
            vec![Annotation {
                entry: Utc.ymd(2022, 1, 11).and_hms(7, 41, 12),
                description: String::from("note:dp"),
            }]
        );
    }
}
