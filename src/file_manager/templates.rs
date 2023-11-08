use askama::Template;

use crate::file_server::models::Directory;

#[derive(Debug, Template)]
#[template(path = "index.html")]
pub struct HomePageTemplate {
    pub css_content: String,
}

#[derive(Debug, Template)]
#[template(path = "program-list.html", escape = "none")]
pub struct ProgramListTemplate {
    pub base_dir: Directory,
}
