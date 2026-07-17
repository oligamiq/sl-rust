#![cfg(not(target_os = "wasi"))]

use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[cfg(windows)]
const BUSYBOX_DIR: &str = r"C:\bin\busybox";

fn run_ours(args: &[&str]) -> String {
    let output = Command::new("cargo")
        .args(&["run", "-q", "-p", "wasibox-core", "--"])
        .args(args)
        .output()
        .expect("Failed to run our utility via cargo run");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[cfg(windows)]
fn run_busybox(util: &str, args: &[&str]) -> String {
    let exe = format!(r"{}\{}.exe", BUSYBOX_DIR, util);
    let output = Command::new(&exe)
        .args(args)
        .output()
        .expect(&format!("Failed to run busybox {}", util));
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[cfg(not(windows))]
fn run_busybox(util: &str, args: &[&str]) -> String {
    let output = Command::new("busybox")
        .arg(util)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("Failed to run busybox {util}: {e}"));

    assert!(
        output.status.success(),
        "busybox {util} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn test_ls_al_compatibility() {
    let tmp = tempdir().unwrap();
    let tmp_path = tmp.path();

    // Create a predictable environment
    fs::write(tmp_path.join("file.txt"), "hello").unwrap();
    fs::create_dir(tmp_path.join("subdir")).unwrap();

    let bb_out = run_busybox("ls", &["-al", tmp_path.to_str().unwrap()]);
    let our_out = run_ours(&["ls", "-al", tmp_path.to_str().unwrap()]);

    // Check for essential GNU-like elements in our output
    assert!(
        our_out.contains("total"),
        "Output should contain 'total' line"
    );
    assert!(our_out.contains("."), "Output should contain '.' entry");
    assert!(our_out.contains(".."), "Output should contain '..' entry");
    assert!(
        our_out.contains("file.txt"),
        "Output should contain 'file.txt'"
    );
    assert!(our_out.contains("subdir"), "Output should contain 'subdir'");

    // Compare line counts (approximate compatibility)
    let bb_lines = bb_out.lines().count();
    let our_lines = our_out.lines().count();
    assert_eq!(
        our_lines, bb_lines,
        "Line count mismatch between BusyBox and our ls"
    );
}

#[test]
fn test_mkdir_p_compatibility() {
    let tmp = tempdir().unwrap();
    let nested = tmp.path().join("a/b/c");
    let nested_str = nested.to_str().unwrap();

    // Use our mkdir -p
    run_ours(&["mkdir", "-p", nested_str]);
    assert!(nested.exists() && nested.is_dir());

    // Clean up and verify BusyBox does the same (sanity check)
    fs::remove_dir_all(tmp.path().join("a")).unwrap();
    run_busybox("mkdir", &["-p", nested_str]);
    assert!(nested.exists() && nested.is_dir());
}

#[test]
fn test_uname_a_compatibility() {
    let bb_out = run_busybox("uname", &["-a"]);
    let our_out = run_ours(&["uname", "-a"]);

    assert!(!bb_out.trim().is_empty());
    assert!(!our_out.trim().is_empty());
    assert!(
        our_out.contains(std::env::consts::ARCH),
        "uname output does not contain architecture: {our_out:?}"
    );

    #[cfg(target_os = "linux")]
    assert!(
        our_out.contains("Linux"),
        "uname output does not contain Linux: {our_out:?}"
    );

    #[cfg(windows)]
    assert!(
        our_out.contains("Windows"),
        "uname output does not contain Windows: {our_out:?}"
    );
}

#[test]
fn test_cat_compatibility() {
    let tmp = tempdir().unwrap();
    let file = tmp.path().join("test.txt");
    fs::write(&file, "compatibility test").unwrap();
    let file_str = file.to_str().unwrap();

    let bb_out = run_busybox("cat", &[file_str]);
    let our_out = run_ours(&["cat", file_str]);

    assert_eq!(our_out.trim(), bb_out.trim());
}
