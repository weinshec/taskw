use chrono::{DateTime, TimeZone, Utc};
use serde::{self, Deserialize, Deserializer, Serializer};

const FORMAT: &'static str = "%Y%m%dT%H%M%SZ";

pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = format!("{}", date.format(FORMAT));
    serializer.serialize_str(&s)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Utc.datetime_from_str(&s, FORMAT)
        .map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(serde::Serialize, serde::Deserialize)]
    struct TestDateTime {
        #[serde(with = "super")]
        datetime: DateTime<Utc>,
    }

    #[test]
    fn deserialize_taskwarrior_datetime_format() {
        let json_str = r#"{"datetime":"20220110T171619Z"}"#;
        let datetime = Utc.ymd(2022, 1, 10).and_hms(17, 16, 19);

        let testdt: TestDateTime =
            serde_json::from_str(json_str).expect("deserialization succeeded");
        assert_eq!(testdt.datetime, datetime);
    }

    #[test]
    fn serialize_taskwarrior_datetime_format() {
        let json_str = r#"{"datetime":"20220110T171619Z"}"#;
        let testdt = TestDateTime {
            datetime: Utc.ymd(2022, 1, 10).and_hms(17, 16, 19),
        };
        assert_eq!(json_str, serde_json::to_string(&testdt).expect(""));
    }
}
