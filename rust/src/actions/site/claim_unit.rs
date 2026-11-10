use super::*;

#[test]
fn refuses_a_project_already_claimed_until_released() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let other = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let held = claim_site(root.path())?;
    assert!(matches!(claim_site(root.path()), Err(Error::Usage(_))));
    claim_site(other.path())?;
    drop(held);
    claim_site(root.path())?;
    Ok(())
}
