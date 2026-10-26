use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::actions::path::contain::contain_path;
use crate::models::error::Error;
use crate::models::skill::Skill;
use crate::settings::CONFIG_FILE;

/// Read every `SKILL.md` the `skills` globs match, relative to `root`, sorted by path.
/// Each needs a frontmatter `name` (lowercase, hyphenated) and `description`.
pub fn load_skills(root: &Path, patterns: &[String]) -> crate::Result<Vec<Skill>> {
    let fail = |message: String| Error::Config {
        path: root.join(CONFIG_FILE),
        message,
    };
    let options = glob::MatchOptions {
        require_literal_separator: true,
        ..glob::MatchOptions::new()
    };
    let mut paths: Vec<PathBuf> = vec![];
    for pattern in patterns {
        let full = root.join(pattern).to_string_lossy().into_owned();
        let matches = glob::glob_with(&full, options)
            .map_err(|error| fail(format!("skills: \"{pattern}\": {error}")))?;
        for path in matches {
            let path = path.map_err(|error| fail(format!("skills: {error}")))?;
            // A skill is published as is, so a pattern may not reach past the project,
            // though dot-folders such as `.skills/` are allowed
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .into_owned();
            let real = contain_path(root, &relative, true)
                .map_err(|message| fail(format!("skills: {message}")))?;
            paths.push(real);
        }
    }
    paths.sort();
    paths.dedup();
    let mut skills: Vec<Skill> = vec![];
    for path in paths {
        let skill = read_skill(&path)?;
        if skills.iter().any(|other| other.name == skill.name) {
            return Err(fail(format!(
                "skills: two skills are named \"{}\"",
                skill.name
            )));
        }
        skills.push(skill);
    }
    Ok(skills)
}

#[derive(Deserialize)]
struct Frontmatter {
    name: String,
    description: String,
}

fn read_skill(path: &Path) -> crate::Result<Skill> {
    let source = std::fs::read(path).map_err(Error::io(path))?;
    let invalid = || Error::Config {
        path: path.to_owned(),
        message: "a published skill needs a frontmatter `name` (lowercase, hyphenated, \
                  at most 64 characters) and a `description` (at most 1024)"
            .to_owned(),
    };
    let text = String::from_utf8_lossy(&source).replace("\r\n", "\n");
    let frontmatter = text
        .strip_prefix("---\n")
        .and_then(|rest| rest.split_once("\n---"))
        .map(|(frontmatter, _)| frontmatter)
        .ok_or_else(invalid)?;
    let parsed: Frontmatter =
        serde_saphyr::from_str(frontmatter).map_err(|_| invalid())?;
    let named = parsed.name.len() <= 64
        && parsed.name.split('-').all(|word| {
            !word.is_empty()
                && word
                    .chars()
                    .all(|char| char.is_ascii_lowercase() || char.is_ascii_digit())
        });
    let described = !parsed.description.is_empty() && parsed.description.len() <= 1024;
    if !named || !described {
        return Err(invalid());
    }
    Ok(Skill {
        name: parsed.name,
        description: parsed.description,
        path: path.to_owned(),
        source,
    })
}

#[cfg(test)]
#[path = "load_unit.rs"]
mod unit;
