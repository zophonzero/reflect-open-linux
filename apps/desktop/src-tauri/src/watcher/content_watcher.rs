//! Filter Linux read notifications before they reach the debouncer cache.
//!
//! Walking an uncached directory opens and closes it. Feeding those events
//! back into the pruned cache can repeat the walk indefinitely; delivering
//! them to the frontend also makes indexing trigger another reconciliation.

use std::path::Path;

use notify::event::{AccessKind, AccessMode};
use notify::{
    Config, EventHandler, EventKind, RecommendedWatcher, RecursiveMode, Result, Watcher,
    WatcherKind,
};

pub struct ContentWatcher(RecommendedWatcher);

impl Watcher for ContentWatcher {
    fn new<F: EventHandler>(mut handler: F, config: Config) -> Result<Self> {
        RecommendedWatcher::new(
            move |result: Result<notify::Event>| {
                if let Ok(event) = &result {
                    if !event.need_rescan()
                        && matches!(event.kind, EventKind::Access(_))
                        && !matches!(
                            event.kind,
                            EventKind::Access(AccessKind::Close(AccessMode::Write))
                        )
                    {
                        return;
                    }
                }
                handler.handle_event(result);
            },
            config,
        )
        .map(Self)
    }

    fn watch(&mut self, path: &Path, mode: RecursiveMode) -> Result<()> {
        self.0.watch(path, mode)
    }

    fn unwatch(&mut self, path: &Path) -> Result<()> {
        self.0.unwatch(path)
    }

    fn configure(&mut self, config: Config) -> Result<bool> {
        self.0.configure(config)
    }

    fn kind() -> WatcherKind {
        RecommendedWatcher::kind()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, sync::mpsc, time::Duration};

    #[test]
    fn reading_notes_and_walking_hidden_directories_do_not_emit_changes() {
        let root = tempfile::tempdir().unwrap();
        let note = root.path().join("note.md");
        let hidden = root.path().join(".reflect");
        fs::create_dir(&hidden).unwrap();
        fs::write(&note, "original").unwrap();
        let (sender, receiver) = mpsc::channel();
        let mut watcher = ContentWatcher::new(sender, Config::default()).unwrap();
        watcher
            .watch(root.path(), RecursiveMode::Recursive)
            .unwrap();

        for _iteration in 0..10 {
            fs::read(&note).unwrap();
            assert_eq!(fs::read_dir(&hidden).unwrap().count(), 0);
            assert_eq!(fs::read_dir(root.path()).unwrap().count(), 2);
        }
        assert!(matches!(
            receiver.recv_timeout(Duration::from_millis(200)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ));

        fs::write(&note, "changed").unwrap();
        let event = receiver
            .recv_timeout(Duration::from_secs(5))
            .unwrap()
            .unwrap();
        assert!(event.paths.contains(&note));
        assert!(matches!(
            event.kind,
            EventKind::Modify(_) | EventKind::Access(AccessKind::Close(AccessMode::Write))
        ));
    }
}
