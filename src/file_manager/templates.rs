use askama::Template;
use file_server_core::{Directory, DirectoryEntry};

#[derive(Debug, Template)]
#[template(path = "index.html")]
pub struct HomePageTemplate {
    pub css_content: String,
}

#[derive(Debug, Template)]
#[template(path = "program-list.html", escape = "none")]
pub struct ProgramListTemplate {
    pub base_dir: DirectoryTemplate,
}

#[derive(Debug)]
pub struct DirectoryTemplate {
    pub name: String,
    // TODO: Make optional to represent unvisited state (Or use enum?)
    pub entries: Vec<DirectoryEntryTemplate>,
    pub path: String,
}

#[derive(Debug, Template)]
#[template(path = "directory-entry.html", escape = "none")]
pub enum DirectoryEntryTemplate {
    Directory(DirectoryTemplate),
    File { name: String, path: String },
}

impl From<Directory> for DirectoryTemplate {
    fn from(value: Directory) -> Self {
        let entries = value.entries
            .into_iter()
            .map(DirectoryEntryTemplate::from)
            .collect();

        Self { 
            name: value.name,
            entries,
            path: value.path.to_str().unwrap_or("").to_owned(),
        }
    }
}

impl From<DirectoryEntry> for DirectoryEntryTemplate {
    fn from(value: DirectoryEntry) -> Self {
        match value {
            DirectoryEntry::Directory(directory) => {
                let directory = DirectoryTemplate::from(directory);
                Self::Directory(directory) 
            },
            DirectoryEntry::File { name, path } => {
                let path = path.to_str().unwrap_or("").to_owned();
                Self::File { name, path }
            },
        }
    }
}
