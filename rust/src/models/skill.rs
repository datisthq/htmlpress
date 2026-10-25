use std::path::PathBuf;

/// A `SKILL.md` published for agents.
#[derive(Debug, Clone, PartialEq)]
pub struct Skill {
    /// The frontmatter `name`: lowercase words joined by hyphens.
    pub name: String,
    pub description: String,
    /// The file it was read from.
    pub path: PathBuf,
    /// The file's bytes, which the index's digest covers.
    pub source: Vec<u8>,
}
