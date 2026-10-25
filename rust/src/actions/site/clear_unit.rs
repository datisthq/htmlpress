use super::*;
use crate::actions::site::build::build_site;

#[test]
fn removes_the_output_once() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    std::fs::write(root.join("index.html"), "<h1>Home</h1>").map_err(Error::io(root))?;
    let options = BuildOptions::new(root);
    build_site(&options)?;
    assert!(clear_site(&options)?);
    assert!(!options.output.exists() && root.join("index.html").is_file());
    assert!(!clear_site(&options)?);
    Ok(())
}

#[test]
fn refuses_an_output_that_holds_the_project() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let options = BuildOptions {
        output: root.path().to_owned(),
        ..BuildOptions::new(root.path().join("docs"))
    };
    assert!(matches!(clear_site(&options), Err(Error::Usage(_))));
    assert!(root.path().exists());
    Ok(())
}
