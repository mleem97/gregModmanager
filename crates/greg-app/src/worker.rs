//! Background jobs: run blocking work on threads, stream progress/logs.

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

/// Progress/log messages from a worker.
#[derive(Debug)]
pub enum JobMessage {
    /// 0.0–1.0 progress fraction.
    Progress(f32),
    /// Status/log line.
    Log(String),
    /// Status line replacing the header.
    Status(String),
    /// Terminal outcome.
    Done(JobOutcome),
}

/// Terminal outcome.
#[derive(Debug)]
pub struct JobOutcome {
    /// Success flag.
    pub success: bool,
    /// Message.
    pub message: String,
}

/// Handle to a running job.
pub struct JobHandle {
    rx: Receiver<JobMessage>,
    cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl JobHandle {
    /// Spawns `work` on a background thread.
    pub fn spawn(work: impl FnOnce(JobSink) -> JobOutcome + Send + 'static) -> Self {
        let (tx, rx) = mpsc::channel();
        let cancelled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let flag = std::sync::Arc::clone(&cancelled);
        let handle = thread::spawn(move || {
            let sink = JobSink {
                tx,
                cancelled: flag,
            };
            let outcome = work(sink.clone());
            let _ = sink.tx.send(JobMessage::Done(outcome));
        });
        Self {
            rx,
            cancelled,
            handle: Some(handle),
        }
    }

    /// Drains pending messages (call once per frame).
    pub fn drain(&mut self) -> Vec<JobMessage> {
        let mut out = Vec::new();
        while let Ok(msg) = self.rx.try_recv() {
            out.push(msg);
        }
        out
    }

    /// Requests cancellation.
    pub fn cancel(&self) {
        self.cancelled
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// True when the thread finished.
    pub fn is_finished(&self) -> bool {
        self.handle.as_ref().is_some_and(|h| h.is_finished())
    }
}

impl Drop for JobHandle {
    fn drop(&mut self) {
        self.cancel();
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// Progress/log sink handed to workers.
#[derive(Debug, Clone)]
pub struct JobSink {
    tx: Sender<JobMessage>,
    cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl JobSink {
    /// Reports progress.
    pub fn progress(&self, fraction: f32) {
        let _ = self.tx.send(JobMessage::Progress(fraction.clamp(0.0, 1.0)));
    }

    /// Reports a log line.
    pub fn log(&self, line: impl Into<String>) {
        let _ = self.tx.send(JobMessage::Log(line.into()));
    }

    /// Reports a status line.
    pub fn status(&self, line: impl Into<String>) {
        let _ = self.tx.send(JobMessage::Status(line.into()));
    }

    /// Cancellation flag.
    pub fn cancelled(&self) -> &std::sync::Arc<std::sync::atomic::AtomicBool> {
        &self.cancelled
    }
}
