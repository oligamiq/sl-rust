use lazy_static::lazy_static;
use std::env;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

// Cargo runs tests in parallel by default.
// Modifying environment variables in a multithreaded test environment
// causes race conditions and is inherently unsafe in Rust >= 1.80.
// We use a global Mutex to serialize our env var tests to avoid cross-test pollution.
lazy_static! {
    static ref ENV_MUTEX: Mutex<()> = Mutex::new(());
}

/// Helper function to parse terminal size from env vars exactly like `terminal::sys::init`
/// for the WASI target in `sl-rust`.
fn get_terminal_size_from_env() -> (u16, u16) {
    let w = env::var("COLUMNS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(80);
    let h = env::var("LINES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(24);
    (w, h)
}

#[test]
fn test_read_env_vars_single_thread() {
    let _lock = ENV_MUTEX.lock().unwrap();

    // Safe to modify in a controlled environment (though still technically unsafe in multithreaded rust >= 1.80).
    // We use `unsafe` block if required by the Rust version, but `env::set_var` is safe in Rust < 1.80.
    // For wider compatibility, we just use the standard call. If the compiler complains, we can adjust.
    unsafe {
        env::set_var("COLUMNS", "120");
        env::set_var("LINES", "40");
    }

    let (w, h) = get_terminal_size_from_env();
    assert_eq!(w, 120);
    assert_eq!(h, 40);

    // Clean up
    unsafe {
        env::remove_var("COLUMNS");
        env::remove_var("LINES");
    }
}

#[test]
fn test_env_vars_across_threads_sharing() {
    let _lock = ENV_MUTEX.lock().unwrap();

    // Set initial values
    unsafe {
        env::set_var("COLUMNS", "100");
        env::set_var("LINES", "30");
    }

    // Prove that a spawned thread reads the same environment variables.
    // Environment variables are process-global.
    let handle = thread::spawn(|| {
        let (w, h) = get_terminal_size_from_env();
        assert_eq!(w, 100);
        assert_eq!(h, 30);
    });

    handle.join().unwrap();

    // Clean up
    unsafe {
        env::remove_var("COLUMNS");
        env::remove_var("LINES");
    }
}

#[test]
fn test_env_vars_across_threads_race_condition() {
    let _lock = ENV_MUTEX.lock().unwrap();

    // Start with a base value
    unsafe {
        env::set_var("COLUMNS", "80");
        env::set_var("LINES", "24");
    }

    // We spawn a thread that constantly reads the env vars.
    let reader_handle = thread::spawn(|| {
        for _ in 0..100 {
            let (w, h) = get_terminal_size_from_env();
            // It should be either the original or the modified value.
            assert!((w == 80 && h == 24) || (w == 200 && h == 50));
            thread::sleep(Duration::from_millis(1));
        }
    });

    // While the other thread is reading, we modify the env vars.
    // This demonstrates the race condition. The reader might catch
    // intermediate states if env var updating wasn't atomic (which it isn't at OS level usually,
    // though std::env tries to be safe, set_var is unsafe for a reason in 1.80+).
    thread::sleep(Duration::from_millis(10));
    unsafe {
        env::set_var("COLUMNS", "200");
        env::set_var("LINES", "50");
    }

    reader_handle.join().unwrap();

    // Verify the main thread sees the new value
    let (w, h) = get_terminal_size_from_env();
    assert_eq!(w, 200);
    assert_eq!(h, 50);

    // Clean up
    unsafe {
        env::remove_var("COLUMNS");
        env::remove_var("LINES");
    }
}

#[test]
fn test_env_vars_observed_by_waiting_thread() {
    let _lock = ENV_MUTEX.lock().unwrap();

    // Set initial values
    unsafe {
        env::set_var("COLUMNS", "80");
        env::set_var("LINES", "24");
    }

    use std::sync::{Arc, Barrier};
    // We use barriers to strictly control the timing of execution between the main and spawned thread.
    let barrier_read_initial = Arc::new(Barrier::new(2));
    let barrier_modified = Arc::new(Barrier::new(2));

    let b_read_initial = Arc::clone(&barrier_read_initial);
    let b_modified = Arc::clone(&barrier_modified);

    let handle = thread::spawn(move || {
        // Step 1: Read the initial environment variables
        let (w1, h1) = get_terminal_size_from_env();
        assert_eq!(w1, 80);
        assert_eq!(h1, 24);

        // Signal to the main thread that initial read is complete, and wait here
        b_read_initial.wait();

        // Block here until the main thread changes the environment variables
        b_modified.wait();

        // Step 3: Observe the new values set by the main thread while this thread was waiting
        let (w2, h2) = get_terminal_size_from_env();
        assert_eq!(w2, 120);
        assert_eq!(h2, 40);
    });

    // Wait for the spawned thread to finish its initial read
    barrier_read_initial.wait();

    // Step 2: Main thread modifies the environment variables.
    // The spawned thread already exists and is currently waiting.
    unsafe {
        env::set_var("COLUMNS", "120");
        env::set_var("LINES", "40");
    }

    // Signal the waiting spawned thread to proceed
    barrier_modified.wait();

    handle.join().unwrap();

    // Clean up
    unsafe {
        env::remove_var("COLUMNS");
        env::remove_var("LINES");
    }
}
