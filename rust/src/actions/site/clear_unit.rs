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
fn removes_the_cache_too() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let cache = root.path().join(CACHE_DIR);
    std::fs::create_dir_all(&cache).map_err(Error::io(&cache))?;
    assert!(clear_site(&BuildOptions::new(root.path()))?);
    assert!(!cache.exists());
    Ok(())
}

#[test]
fn refuses_while_start_serves_the_project() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let options = BuildOptions::new(root.path());
    std::fs::create_dir_all(&options.output).map_err(Error::io(&options.output))?;
    let _serving = claim_site(root.path())?;
    assert!(matches!(clear_site(&options), Err(Error::Usage(_))));
    assert!(options.output.exists());
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

#[test]
fn refuses_a_folder_that_is_no_build_output() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    for name in [".git", ".htmlpress", "../keep"] {
        let folder = root.join("site").join(name);
        std::fs::create_dir_all(&folder).map_err(Error::io(&folder))?;
        std::fs::write(folder.join("file"), "mine").map_err(Error::io(&folder))?;
        let options = BuildOptions {
            output: folder.clone(),
            ..BuildOptions::new(root.join("site"))
        };
        assert!(
            matches!(clear_site(&options), Err(Error::Usage(_))),
            "{name}"
        );
        assert!(folder.join("file").is_file(), "{name}");
    }
    Ok(())
}
