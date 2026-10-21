use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender, SyncSender},
    },
    thread,
    time::Duration,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Upsert(PathBuf),
    Remove(PathBuf),
}

pub struct Watcher {
    rx: Receiver<Event>,
    stop: Option<Sender<()>>,
    thread: Option<thread::JoinHandle<()>>,
    overflowed: Arc<AtomicBool>,
}
impl Watcher {
    pub fn start(root: PathBuf) -> Self {
        Self::start_inner(root, 64)
    }
    fn start_inner(root: PathBuf, capacity: usize) -> Self {
        let (tx, rx) = mpsc::sync_channel(capacity);
        let (stop, stop_rx) = mpsc::channel();
        let overflowed = Arc::new(AtomicBool::new(false));
        let signal = Arc::clone(&overflowed);
        let thread = thread::spawn(move || poll(root, tx, stop_rx, signal));
        Self {
            rx,
            stop: Some(stop),
            thread: Some(thread),
            overflowed,
        }
    }
    pub fn overflowed(&self) -> bool {
        self.overflowed.swap(false, Ordering::AcqRel)
    }
    #[cfg(test)]
    pub(crate) fn trigger_overflow(&self) {
        self.overflowed.store(true, Ordering::Release);
    }
    pub fn drain(&self) -> Vec<Event> {
        let mut latest = BTreeMap::new();
        while let Ok(event) = self.rx.try_recv() {
            let path = match &event {
                Event::Upsert(p) | Event::Remove(p) => p.clone(),
            };
            latest.insert(path, event);
        }
        latest.into_values().collect()
    }
}
impl Drop for Watcher {
    fn drop(&mut self) {
        self.stop.take().map(|s| s.send(()));
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
fn files(root: &Path) -> (BTreeMap<PathBuf, std::time::SystemTime>, bool) {
    let mut out = BTreeMap::new();
    let mut complete = true;
    let mut dirs = vec![root.to_path_buf()];
    while let Some(dir) = dirs.pop() {
        let Ok(entries) = fs::read_dir(dir) else {
            complete = false;
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                dirs.push(p);
                continue;
            }
            if matches!(
                p.extension().and_then(|x| x.to_str()),
                Some("shelly" | "shl")
            ) {
                match e.metadata().and_then(|m| m.modified()) {
                    Ok(m) => {
                        out.insert(p, m);
                    }
                    Err(_) => {
                        complete = false;
                    }
                }
            }
        }
    }
    (out, complete)
}
fn poll(root: PathBuf, tx: SyncSender<Event>, stop: Receiver<()>, overflowed: Arc<AtomicBool>) {
    let (mut old, _) = files(&root);
    loop {
        if stop.recv_timeout(Duration::from_millis(40)).is_ok() {
            break;
        }
        let (new, complete) = files(&root);
        for (p, m) in &new {
            if old.get(p) != Some(m) && tx.try_send(Event::Upsert(p.clone())).is_err() {
                overflowed.store(true, Ordering::Release);
            }
        }
        // A partial scan cannot distinguish an error from a deletion.
        if complete {
            for p in old.keys() {
                if !new.contains_key(p) && tx.try_send(Event::Remove(p.clone())).is_err() {
                    overflowed.store(true, Ordering::Release);
                }
            }
            old = new;
        } else {
            // Preserve the last known state for paths hidden by transient errors.
            for (p, m) in new {
                old.insert(p, m);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detects_create_and_delete() {
        let root = std::env::temp_dir().join(format!("shelly-watch-{}", std::process::id()));
        let _ = fs::create_dir_all(&root);
        let w = Watcher::start(root.clone());
        std::thread::sleep(Duration::from_millis(80));
        let p = root.join("a.shl");
        fs::write(&p, "x").unwrap();
        std::thread::sleep(Duration::from_millis(100));
        assert!(
            w.drain()
                .iter()
                .any(|e| matches!(e, Event::Upsert(x) if x == &p))
        );
        fs::remove_file(&p).unwrap();
        std::thread::sleep(Duration::from_millis(100));
        assert!(
            w.drain()
                .iter()
                .any(|e| matches!(e, Event::Remove(x) if x == &p))
        );
        let _ = fs::remove_dir(&root);
    }
}
