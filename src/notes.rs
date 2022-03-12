use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// A notes file associated with a taskwarrior task
pub struct NotesFile {
    path: PathBuf,
    header: Option<YamlMeta>,
    content: String,
}

impl NotesFile {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            header: None,
            content: String::new(),
        }
    }

    pub fn with_header(mut self, header: YamlMeta) -> Self {
        self.header = Some(header);
        self
    }

    pub fn with_content(mut self, content: &str) -> Self {
        self.content = content.to_string();
        self
    }

    pub fn write(&self) -> Result<(), &'static str> {
        let mut file =
            std::fs::File::create(&self.path).map_err(|_| "Cannot open file for writing")?;
        if let Some(header) = &self.header {
            write!(file, "{}---\n\n", header).map_err(|_| "Cannot write header to file")?;
        }
        write!(file, "{}", self.content).map_err(|_| "Cannot write content to file")?;
        Ok(())
    }

    pub fn open(path: &Path) -> Result<Self, &'static str> {
        let document = std::fs::read_to_string(path).map_err(|_| "Cannot read notes file")?;

        let (header, content) = match split_yaml_header(&document) {
            Some((yaml_str, content_str)) => (
                YamlMeta::from_str(yaml_str).ok(),
                content_str.trim().to_string(),
            ),
            None => (None, document.trim().to_string()),
        };

        Ok(Self {
            path: path.to_path_buf(),
            header,
            content,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct YamlMeta {
    title: String,
    date: NaiveDate,
    keywords: Vec<String>,

    #[serde(flatten)]
    unknown_fields: HashMap<String, Value>,
}

impl YamlMeta {
    pub fn new(title: &str, date: NaiveDate) -> Self {
        Self {
            title: title.to_string(),
            date,
            keywords: vec![],
            unknown_fields: HashMap::new(),
        }
    }
}

impl FromStr for YamlMeta {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_yaml::from_str(s).map_err(|_| "Deserialization succeeds")
    }
}

impl std::fmt::Display for YamlMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let yaml_str = serde_yaml::to_string(&self).map_err(|_| std::fmt::Error {})?;
        write!(f, "{}", yaml_str)
    }
}

fn split_yaml_header(s: &str) -> Option<(&str, &str)> {
    let mut tokens = s.trim().splitn(3, "---").skip(1);
    Some((tokens.next()?.trim(), tokens.next()?.trim()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{tempdir, NamedTempFile};

    const YAML_STR: &str = "\
                            title: Complex note title\n\
                            date: 2022-02-18\n\
                            keywords:\n  \
                              - projectX\n  \
                              - withQuotes\n\
                            author: \"That's me\"";

    #[test]
    fn deserialize_simple_yaml_meta() {
        let yaml = YamlMeta::from_str(YAML_STR).expect("Deserialization succeeds");
        assert_eq!(yaml.title, "Complex note title");
        assert_eq!(yaml.date, NaiveDate::from_ymd(2022, 2, 18));
        assert_eq!(
            yaml.keywords,
            vec!["projectX".to_string(), "withQuotes".to_string()]
        );
        assert_eq!(yaml.unknown_fields["author"], "That's me");
    }

    #[test]
    fn split_yaml_header_with_valid_yaml_header() {
        let content_str = "## Document Headline\n\nand some content";
        let header_str = format!("---\n{}\n---", YAML_STR);
        let document_str = format!("{}\n\n{}", header_str, content_str);

        let (header, content) = split_yaml_header(&document_str).expect("splitting succeeds");
        assert_eq!(header, YAML_STR);
        assert_eq!(content, content_str);
    }

    #[test]
    fn split_yaml_header_without_yaml_header() {
        let content_str = "## Document Headline\n\nand some content";
        let document_str = format!("\n\n{}", content_str);
        assert_eq!(None, split_yaml_header(&document_str));
    }

    #[test]
    fn split_yaml_header_with_invalid_yaml_header() {
        let content_str = "## Document Headline\n\nand some content";
        let document_str = format!("---\nFOOBAR\n---\n\n{}", content_str);

        let (header, content) = split_yaml_header(&document_str).expect("notes file from_str");
        assert_eq!(header, "FOOBAR");
        assert_eq!(content, content_str);
    }

    #[test]
    fn read_notes_files_from_filesystem() {
        let content_str = "## Document Headline\n\nand some content";
        let header_str = format!("---\n{}\n---", YAML_STR);
        let document_str = format!("{}\n\n{}", header_str, content_str);

        let mut test_notes_file = NamedTempFile::new().expect("created tempfile");
        write!(test_notes_file, "{}", document_str).expect("writing tempfile");

        let notes_file = NotesFile::open(test_notes_file.path()).expect("reading notes files");
        assert_eq!(notes_file.content, content_str);
        assert_eq!(notes_file.path, test_notes_file.path());
        assert!(notes_file.header.is_some());
    }

    #[test]
    fn read_notes_files_without_header_from_filesystem() {
        let document_str = "---\n\n## Document Headline\n\nand some content";

        let mut test_notes_file = NamedTempFile::new().expect("created tempfile");
        write!(test_notes_file, "{}", document_str).expect("writing tempfile");

        let notes_file = NotesFile::open(test_notes_file.path()).expect("reading notes files");
        assert_eq!(notes_file.content, document_str);
        assert!(notes_file.header.is_none());
    }

    #[test]
    fn read_notes_files_with_invalid_header_from_filesystem() {
        let content_str = "## Document Headline\n\nand some content";
        let header_str = "---\nFOOBAR\n---";
        let document_str = format!("{}\n\n{}", header_str, content_str);

        let mut test_notes_file = NamedTempFile::new().expect("created tempfile");
        write!(test_notes_file, "{}", document_str).expect("writing tempfile");

        let notes_file = NotesFile::open(test_notes_file.path()).expect("reading notes files");
        assert_eq!(notes_file.content, content_str);
        assert!(notes_file.header.is_none());
    }

    #[test]
    fn create_notes_file_in_filesystem() {
        let temp_dir = tempdir().expect("create temporary directory");
        let file_path = temp_dir.path().join("test_notes_file.md");

        let content_str = "## Document Headline\n\nand some content";
        let yaml_meta = serde_yaml::from_str(YAML_STR).expect("parse yaml meta");

        let notes_file = NotesFile::new(&file_path)
            .with_header(yaml_meta)
            .with_content(content_str);
        notes_file.write().expect("writing notes file succeeds");

        let written_str = std::fs::read_to_string(&file_path).expect("read written file");
        assert_eq!(
            written_str,
            format!("---\n{}\n---\n\n{}", YAML_STR, content_str)
        );
    }
}
