use askama::Template;
use file_server_core::{Directory, DirectoryEntry};

#[derive(Debug, Template)]
#[template(path = "index.html")]
pub struct HomePageTemplate {
    pub css_content: String,
}

#[derive(Debug, Template)]
#[template(path = "program-list.html", escape = "none")]
pub struct ProgramListTemplate<'a> {
    pub base_dir: DirectoryTemplate<'a>,
    pub expanded: bool,
}

#[derive(Debug)]
pub struct DirectoryTemplate<'a> {
    pub name: &'a str,
    // TODO: Make optional to represent unvisited state (Or use enum?)
    pub entries: Vec<DirectoryEntryTemplate<'a>>,
    pub path: String,
    pub expanded: bool,
}

#[derive(Debug, Template)]
#[template(path = "directory-entry.html", escape = "none")]
pub enum DirectoryEntryTemplate<'a> {
    Directory(DirectoryTemplate<'a>),
    File { name: &'a str, path: String },
}

impl<'a> From<&'a Directory> for DirectoryTemplate<'a> {
    fn from(value: &'a Directory) -> Self {
        let entries = value
            .entries
            .iter()
            .map(DirectoryEntryTemplate::from)
            .collect();

        Self {
            name: &value.name,
            entries,
            path: value.path.to_str().unwrap_or("").to_owned(),
            expanded: false,
        }
    }
}

impl<'a> From<&'a DirectoryEntry> for DirectoryEntryTemplate<'a> {
    fn from(value: &'a DirectoryEntry) -> Self {
        match value {
            DirectoryEntry::Directory(directory) => {
                let directory = DirectoryTemplate::from(directory);
                Self::Directory(directory)
            }
            DirectoryEntry::File { name, path } => {
                let path = path.to_str().unwrap_or("").to_owned();
                Self::File { name, path }
            }
        }
    }
}

#[derive(Debug, Template)]
#[template(path = "file-content.html", escape = "none")]
pub struct FileContentTemplate<'a> {
    pub name: &'a str,
    pub path: &'a str,
}
