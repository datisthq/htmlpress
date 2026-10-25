use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::models::config::Config;
use crate::models::page::Page;
use crate::models::skill::Skill;

/// A scanned project: its config, its pages in navigation order and its other files.
#[derive(Debug, Clone)]
pub struct Site {
    pub root: PathBuf,
    pub config: Config,
    /// The site title: `title` from the config, else the project folder's name.
    pub title: String,
    pub pages: Vec<Page>,
    /// Every non-page file, relative to the root with `/` separators, copied as-is.
    pub files: Vec<String>,
    /// Every asset the build uses, by its path under `.htmlpress/`: the built-in ones with
    /// the project's escapes and own components laid over them.
    pub assets: BTreeMap<String, String>,
    /// The `SKILL.md` files the config's `skills` globs match, published for agents.
    pub skills: Vec<Skill>,
}
