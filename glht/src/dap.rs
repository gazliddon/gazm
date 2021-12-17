// need to listen on a rx and tx on a tx
use std::sync::mpsc;

enum Msg {
    Text(String)
}

struct Dap {
    rx : mpsc::Receiver<Msg>,
    tx : mpsc::Sender<Msg>,
}

impl Dap {
    pub fn new() -> Self {
        let (tx,rx) = mpsc::channel();

        Self {
            rx,tx
        }
    }
}
