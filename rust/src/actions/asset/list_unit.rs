use super::*;

#[test]
fn lists_every_built_in_asset_as_not_escaped() {
    let statuses = list_assets(Path::new("/nonexistent"));
    assert_eq!(statuses.len(), ASSETS.len());
    assert!(statuses.iter().all(|status| !status.escaped));
}
