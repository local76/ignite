use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

/// Events emitted by background worker threads to notify the TUI main loop.
#[derive(Debug, Clone)]
pub enum WorkerEvent {
    /// Reports fractional progress (0.0 to 1.0) of a background task.
    Progress(f64),
    /// Signals successful completion with a return message.
    Success(String),
    /// Signals failure with an error message string.
    #[allow(dead_code)]
    Error(String),
}

/// Spawns a mock background worker thread that simulates a slow operations (like a network fetch
/// or file search) and communicates progress updates back to the UI.
pub fn spawn_background_task(tx: Sender<WorkerEvent>) {
    thread::spawn(move || {
        // Simulate a step-by-step task
        for i in 1..=20 {
            if library::lifecycle::foreground::tui_bootstrap::is_app_shutting_down() {
                break;
            }
            thread::sleep(Duration::from_millis(150));
            let progress = i as f64 / 20.0;
            let _ = tx.send(WorkerEvent::Progress(progress));
        }

        thread::sleep(Duration::from_millis(100));
        let _ = tx.send(WorkerEvent::Success(
            "Startup optimization scan completed successfully!".to_string(),
        ));
    });
}

