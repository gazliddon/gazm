use std::sync::mpsc::{self, TryRecvError};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

#[derive(Debug, Clone)]
pub enum TermOutput {
    Text(String),
    Error,
    CtrlC,
    CtrlD,
}

pub struct Term {
    rx: Receiver<TermOutput>,
    command_tx: Sender<bool>,
    pub output: Vec<TermOutput>,
}

impl Term {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let (command_tx, command_rx) = mpsc::channel();
        let _child = thread::spawn(move || {
            Self::do_term(tx, command_rx).unwrap();
        });

        Self {
            rx,
            command_tx,
            output: vec![],
        }
    }

    pub fn quit(&self) {
        self.command_tx.send(true).unwrap();
    }

    pub fn update(&mut self) {
        loop {
            let x = self.rx.try_recv();

            match x {
                Ok(output) => self.output.push(output),
                Err(TryRecvError::Empty) => break,
                _ => panic!("{:?}", x),
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.output.is_empty()
    }

    pub fn flush(&mut self) -> Vec<TermOutput> {
        self.update();
        let ret = self.output.clone();
        self.output = vec![];
        ret
    }

    fn do_term(
        tx: Sender<TermOutput>,
        rx: Receiver<bool>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use rustyline::error::ReadlineError;
        use rustyline::Editor;

        let mut rl = Editor::<()>::new();

        if rl.load_history("history.txt").is_err() {
            println!("No previous history.");
        }

        let _sources: Option<utils::sources::SourceDatabase> = None;

        loop {
            let has_quit = rx.try_recv();

            if let Ok(_) = has_quit {
                break;
            }

            let readline = rl.readline("> ");

            match readline {
                Ok(line) => {
                    tx.send(TermOutput::Text(line.to_string()))?;
                    rl.add_history_entry(line.as_str());
                }

                Err(ReadlineError::Interrupted) => {
                    tx.send(TermOutput::CtrlC)?;
                }

                Err(ReadlineError::Eof) => {
                    tx.send(TermOutput::CtrlD)?;
                }

                Err(_) => {
                    tx.send(TermOutput::Error)?;
                    break;
                }
            }
        }

        rl.save_history("history.txt").unwrap();
        Ok(())
    }
}

