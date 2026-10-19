use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn split_refuses_unstaged_content_before_calling_a_model() {
    let path = std::env::temp_dir().join(format!(
        "git-sage-test-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&path).unwrap();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(&path)
                .output()
                .unwrap()
                .status
                .success()
        );
    };
    git(&["init", "-q"]);
    git(&["config", "user.name", "Test"]);
    git(&["config", "user.email", "test@example.invalid"]);
    fs::write(path.join("file.txt"), "original").unwrap();
    git(&["add", "file.txt"]);
    git(&["commit", "-qm", "initial"]);
    fs::write(path.join("file.txt"), "staged").unwrap();
    git(&["add", "file.txt"]);
    fs::write(path.join("file.txt"), "unstaged").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_git-sage"))
        .arg("--split")
        .current_dir(&path)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("staged and working-tree content"));
    let commits = Command::new("git")
        .args(["rev-list", "--count", "HEAD"])
        .current_dir(&path)
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&commits.stdout).trim(), "1");
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn split_refuses_staged_renames_before_calling_a_model() {
    let path = std::env::temp_dir().join(format!(
        "git-sage-rename-test-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&path).unwrap();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(&path)
                .output()
                .unwrap()
                .status
                .success()
        );
    };
    git(&["init", "-q"]);
    git(&["config", "user.name", "Test"]);
    git(&["config", "user.email", "test@example.invalid"]);
    fs::write(path.join("before.txt"), "original\n").unwrap();
    git(&["add", "before.txt"]);
    git(&["commit", "-qm", "initial"]);
    git(&["mv", "before.txt", "after.txt"]);
    let output = Command::new(env!("CARGO_BIN_EXE_git-sage"))
        .arg("--split")
        .current_dir(&path)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("staged renames"));
    fs::remove_dir_all(path).unwrap();
}
