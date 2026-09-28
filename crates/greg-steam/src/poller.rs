//! Subscription polling + sync orchestration (`SubscriptionPoller` and
//! `WorkshopSyncOrchestrator` port).
//!
//! The poller watches subscribed item ids; the orchestrator turns deltas into
//! download/sync work via caller-provided handlers (game sync itself lives in
//! `greg-loader`, so this crate stays dependency-free in that direction).

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::backend::SteamBackend;
use crate::error::SteamError;

/// Poll interval for subscription deltas.
pub const POLL_INTERVAL: Duration = Duration::from_secs(30);

/// Subscription changes since the last poll.
#[derive(Debug, Clone, Default)]
pub struct SubscriptionDelta {
    /// Newly subscribed ids.
    pub added: Vec<u64>,
    /// Unsubscribed ids.
    pub removed: Vec<u64>,
}

/// Delta callback type.
type DeltaCallback = Box<dyn Fn(SubscriptionDelta) + Send>;

/// Background subscription watcher.
pub struct SubscriptionPoller {
    backend: Arc<dyn SteamBackend>,
    interval: Duration,
    running: Arc<AtomicBool>,
    handle: Mutex<Option<JoinHandle<()>>>,
    on_delta: Arc<Mutex<Option<DeltaCallback>>>,
}

impl SubscriptionPoller {
    /// Creates a stopped poller.
    pub fn new(backend: Arc<dyn SteamBackend>) -> Self {
        Self::with_interval(backend, POLL_INTERVAL)
    }

    /// Creates a stopped poller with a custom interval (tests).
    pub fn with_interval(backend: Arc<dyn SteamBackend>, interval: Duration) -> Self {
        Self {
            backend,
            interval,
            running: Arc::new(AtomicBool::new(false)),
            handle: Mutex::new(None),
            on_delta: Arc::new(Mutex::new(None)),
        }
    }

    /// Registers the delta handler.
    pub fn on_delta(&self, handler: impl Fn(SubscriptionDelta) + Send + 'static) {
        *self.on_delta.lock().expect("poller lock") = Some(Box::new(handler));
    }

    /// Starts the background thread (idempotent).
    pub fn start(&self) {
        if self.running.swap(true, Ordering::SeqCst) {
            return;
        }
        let backend = Arc::clone(&self.backend);
        let running = Arc::clone(&self.running);
        let interval = self.interval;
        let on_delta = Arc::clone(&self.on_delta);
        let mut known: HashSet<u64> = HashSet::new();
        let mut first = true;
        let handle = thread::spawn(move || {
            while running.load(Ordering::SeqCst) {
                match backend_subscribed(&backend) {
                    Ok(current) => {
                        if first {
                            known = current;
                            first = false;
                        } else {
                            let added: Vec<u64> = current.difference(&known).copied().collect();
                            let removed: Vec<u64> = known.difference(&current).copied().collect();
                            known = current;
                            if !added.is_empty() || !removed.is_empty() {
                                if let Some(handler) =
                                    on_delta.lock().expect("poller lock").as_ref()
                                {
                                    handler(SubscriptionDelta { added, removed });
                                }
                            }
                        }
                    }
                    Err(_) => {
                        // Steam hiccup — keep the last known set.
                    }
                }
                thread::sleep(interval);
            }
        });
        *self.handle.lock().expect("poller lock") = Some(handle);
    }

    /// Stops the background thread (idempotent).
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.handle.lock().expect("poller lock").take() {
            let _ = handle.join();
        }
    }

    /// True while polling.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

impl Drop for SubscriptionPoller {
    fn drop(&mut self) {
        self.stop();
    }
}

fn backend_subscribed(backend: &Arc<dyn SteamBackend>) -> Result<HashSet<u64>, SteamError> {
    // Subscribed ids are derived from the first subscribed page plus install
    // info; the backend exposes the authoritative set via `own_ids`-style
    // listing — here we approximate with a bounded page walk.
    use crate::backend::{PageQuery, QueryKind};
    let cancel = AtomicBool::new(false);
    let mut ids = HashSet::new();
    for page in 1..=4u32 {
        let result = backend.page(
            &PageQuery {
                kind: QueryKind::Subscribed,
                page,
                tag: None,
            },
            &cancel,
        )?;
        if result.items.is_empty() {
            break;
        }
        ids.extend(result.items.iter().map(|i| i.published_file_id));
        if result.items.len() as u32 >= result.total {
            break;
        }
    }
    Ok(ids)
}

/// Status events emitted by the orchestrator.
#[derive(Debug, Clone)]
pub enum SyncEvent {
    /// Polling started/stopped.
    Poller(bool),
    /// Download of an item started.
    DownloadStarted(u64),
    /// Sync of a downloaded item started.
    SyncStarted(u64),
    /// Item fully processed.
    Complete(u64),
    /// Item removed (unsubscribed).
    Removed(u64),
    /// Non-fatal warning.
    Warning(String),
    /// Informational message.
    Info(String),
}

/// Turns subscription deltas into download + sync work.
///
/// `download` fetches the item into a staging dir, `sync` installs it into
/// the game (both provided by the composition root, e.g. `greg-loader`).
pub struct WorkshopSyncOrchestrator {
    poller: SubscriptionPoller,
    status: Arc<Mutex<Vec<SyncEvent>>>,
}

impl WorkshopSyncOrchestrator {
    /// Creates the orchestrator (poller stopped).
    pub fn new(backend: Arc<dyn SteamBackend>) -> Self {
        Self {
            poller: SubscriptionPoller::new(backend),
            status: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Starts polling; deltas are downloaded and synced via the handlers.
    pub fn start(
        &self,
        download: impl Fn(u64) -> Result<std::path::PathBuf, String> + Send + Sync + 'static,
        sync: impl Fn(u64, &std::path::Path) -> Result<(), String> + Send + Sync + 'static,
        on_event: impl Fn(SyncEvent) + Send + Sync + 'static,
    ) {
        // Shared ownership keeps the poller callback `'static`.
        let handlers = Arc::new(Handlers {
            download,
            sync,
            on_event,
            status: Arc::clone(&self.status),
        });
        self.poller.on_delta(move |delta| {
            for id in delta.added {
                handlers.emit(SyncEvent::DownloadStarted(id));
                match (handlers.download)(id) {
                    Ok(dir) => {
                        handlers.emit(SyncEvent::SyncStarted(id));
                        match (handlers.sync)(id, &dir) {
                            Ok(()) => handlers.emit(SyncEvent::Complete(id)),
                            Err(e) => handlers
                                .emit(SyncEvent::Warning(format!("sync of {id} failed: {e}"))),
                        }
                    }
                    Err(e) => {
                        handlers.emit(SyncEvent::Warning(format!("download of {id} failed: {e}")))
                    }
                }
            }
            for id in delta.removed {
                handlers.emit(SyncEvent::Removed(id));
            }
        });
        self.emit(SyncEvent::Poller(true));
        self.poller.start();
    }

    /// Stops polling.
    pub fn stop(&self) {
        self.poller.stop();
        self.emit(SyncEvent::Poller(false));
    }

    /// Snapshot of emitted events.
    pub fn events(&self) -> Vec<SyncEvent> {
        self.status.lock().expect("orchestrator lock").clone()
    }

    fn emit(&self, event: SyncEvent) {
        self.status.lock().expect("orchestrator lock").push(event);
    }
}

struct Handlers<D, S, E> {
    download: D,
    sync: S,
    on_event: E,
    status: Arc<Mutex<Vec<SyncEvent>>>,
}

impl<D, S, E> Handlers<D, S, E>
where
    E: Fn(SyncEvent),
{
    fn emit(&self, event: SyncEvent) {
        self.status
            .lock()
            .expect("orchestrator lock")
            .push(event.clone());
        (self.on_event)(event);
    }
}
