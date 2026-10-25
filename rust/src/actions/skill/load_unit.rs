use super::*;

fn write_skill(root: &Path, path: &str, text: &str) -> crate::Result<()> {
    let file = root.join(path);
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent).map_err(Error::io(parent))?;
    }
    std::fs::write(&file, text).map_err(Error::io(&file))
}

#[test]
fn loads_matching_skills_in_path_order() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    let skill = |name: &str| {
        format!(
            "---\nname: {name}\ndescription: Use it.\nlicense: MIT\n---\n\n# {name}\n"
        )
    };
    write_skill(root, "skills/b-two/SKILL.md", &skill("b-two"))?;
    write_skill(root, "skills/a-one/SKILL.md", &skill("a-one"))?;
    write_skill(root, "skills/notes.md", "no frontmatter")?;
    let skills = load_skills(root, &["skills/*/SKILL.md".to_owned()])?;
    let names: Vec<&str> = skills.iter().map(|skill| skill.name.as_str()).collect();
    assert_eq!(names, ["a-one", "b-two"]);
    assert_eq!(skills[0].description, "Use it.");
    assert_eq!(skills[0].source, skill("a-one").into_bytes());
    Ok(())
}

#[test]
fn rejects_a_skill_without_a_valid_name() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    write_skill(
        root,
        "s/SKILL.md",
        "---\nname: Bad Name\ndescription: x\n---\n",
    )?;
    let error = load_skills(root, &["s/SKILL.md".to_owned()]).err();
    assert!(matches!(error, Some(Error::Config { .. })));
    Ok(())
}
