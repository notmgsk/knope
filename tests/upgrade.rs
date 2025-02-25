//! Test the `--upgrade` option.

use std::{
    fs::{copy, read_to_string},
    path::Path,
};

use helpers::*;
use snapbox::cmd::{cargo_bin, Command};

mod helpers;

/// Test running `--upgrade` when there is nothing to upgrade
#[test]
fn upgrade_nothing() {
    // Arrange
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();
    let source_path = Path::new("tests/upgrade/nothing");
    copy(source_path.join("knope.toml"), temp_path.join("knope.toml")).unwrap();

    // Act
    let output_assert = Command::new(cargo_bin!("knope"))
        .arg("--upgrade")
        .current_dir(temp_path)
        .assert();

    // Assert
    output_assert.success().stdout_eq("Nothing to upgrade\n");
    assert().matches_path(
        source_path.join("expected_knope.toml"),
        read_to_string(temp_path.join("knope.toml")).unwrap(),
    );
}
