#![cfg(not(target_os = "wasi"))]

use std::process::Command;
use std::fs;
use tempfile::tempdir;

const BUSYBOX_DIR: &str = r"C:\bin\busybox";

fn run_ours(args: &[&str]) -> String {
    let output = Command::new("cargo")
        .args(&["run", "-q", "-p", "wasibox-core", "--"])
        .args(args)
        .output()
        .expect("Failed to run our utility via cargo run");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn run_busybox(util: &str, args: &[&str]) -> String {
    let exe = format!(r"{}\{}.exe", BUSYBOX_DIR, util);
    let output = Command::new(&exe)
        .args(args)
        .output()
        .expect(&format!("Failed to run busybox {}", util));
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
    assert!(our_out.contains("total"), "Output should contain 'total' line");
    assert!(our_out.contains("."), "Output should contain '.' entry");
    assert!(our_out.contains(".."), "Output should contain '..' entry");
    assert!(our_out.contains("file.txt"), "Output should contain 'file.txt'");
    assert!(our_out.contains("subdir"), "Output should contain 'subdir'");

    // Compare line counts (approximate compatibility)
    let bb_lines = bb_out.lines().count();
    let our_lines = our_out.lines().count();
    assert_eq!(our_lines, bb_lines, "Line count mismatch between BusyBox and our ls");
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
    let _bb_out = run_busybox("uname", &["-a"]);
    let our_out = run_ours(&["uname", "-a"]);
    
    // Both should contain 'Windows' or 'x86_64' on this system
    assert!(our_out.contains("Windows") || our_out.contains("WASI"));
    assert!(our_out.contains("x86_64") || our_out.contains("i686"));
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
