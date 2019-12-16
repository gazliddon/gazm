use notify::{RecommendedWatcher, Watcher, RecursiveMode, DebouncedEvent};
use std::sync::mpsc::{ channel, Receiver };
use std::time::Duration;

pub struct FileWatcher {
    file : String,
    watcher : RecommendedWatcher,
    rx : Receiver<DebouncedEvent>,
}

impl FileWatcher {
    pub fn new(file : &str) -> Self {
        let (tx, rx)  = channel();

        let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(2)).unwrap();
        watcher.watch(file, RecursiveMode::Recursive).unwrap();

        Self { file : file.to_string(), watcher, rx }
    }

    pub fn has_changed(&mut self) -> bool {
        let msg = self.rx.try_recv();
        msg.is_ok()
    }
}

