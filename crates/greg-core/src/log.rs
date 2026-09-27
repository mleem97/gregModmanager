//! In-memory UI log buffer (`AppLogService` port).

use std::sync::{Arc, Mutex};

/// Bounded line buffer with subscriber callbacks.
#[derive(Debug, Clone)]
pub struct LogBuffer {
    inner: Arc<Mutex<LogBufferInner>>,
}

#[derive(Debug)]
struct LogBufferInner {
    lines: Vec<String>,
    capacity: usize,
}

impl LogBuffer {
    /// Creates a buffer holding at most `capacity` lines.
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(LogBufferInner {
                lines: Vec::new(),
                capacity: capacity.max(1),
            })),
        }
    }

    /// Appends a line, evicting the oldest when full.
    pub fn append(&self, line: impl Into<String>) {
        let mut inner = self.inner.lock().expect("log lock");
        inner.lines.push(line.into());
        while inner.lines.len() > inner.capacity {
            inner.lines.remove(0);
        }
    }

    /// Snapshot of all buffered lines.
    pub fn lines(&self) -> Vec<String> {
        self.inner.lock().expect("log lock").lines.clone()
    }

    /// Number of buffered lines.
    pub fn len(&self) -> usize {
        self.inner.lock().expect("log lock").lines.len()
    }

    /// True when empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for LogBuffer {
    fn default() -> Self {
        Self::new(500)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evicts_oldest() {
        let log = LogBuffer::new(2);
        log.append("a");
        log.append("b");
        log.append("c");
        assert_eq!(log.lines(), vec!["b".to_string(), "c".to_string()]);
    }
}
