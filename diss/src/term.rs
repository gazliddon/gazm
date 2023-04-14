use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

#[derive(Debug, Clone)]
pub enum TermOutput {
    Text(String),
    Error,
    CtrlC,
    CtrlD,
}

#[derive(Debug, Clone)]
enum TermControl {
    Quit,
    SetPrompt(String),
}

pub struct Term {
    rx: Receiver<TermOutput>,
    control_tx: Sender<TermControl>,
    pub output: Vec<TermOutput>,
    prompt: String,
}

impl Default for Term {
    fn default() -> Self {
        let history = PathBuf::from_str("history.txt").unwrap();
        let prompt = "> ".to_owned();

        let prompt_copy = prompt.clone();

        let (tx, rx) = channel();
        let (control_tx, control_rx) = channel();

        let _child = thread::spawn(move || {
            Self::do_term(tx, control_rx, prompt, &history).unwrap();
        });

        Self {
            rx,
            control_tx,
            output: vec![],
            prompt: prompt_copy,
        }
    }
}

impl Term {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn quit(&self) {
        self.control_tx.send(TermControl::Quit).unwrap();
    }

    pub fn update(&mut self) {
        loop {
            let x = self.rx.try_recv();

            match x {
                Ok(output) => self.output.push(output),
                Err(TryRecvError::Empty) => break,
                _ => panic!("{x:?}"),
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
        rx: Receiver<TermControl>,
        mut prompt: String,
        history: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut rl = Editor::<()>::new();

        if rl.load_history(history).is_err() {
            println!("No previous history.");
        }

        loop {
            let has_quit = rx.try_recv();

            match has_quit {
                Ok(TermControl::Quit) => break,
                Ok(TermControl::SetPrompt(new_prompt)) => prompt = new_prompt,
                Err(TryRecvError::Empty) => (),
                _ => panic!("{has_quit:?}"),
            }

            let readline = rl.readline(&prompt);

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

        rl.save_history(history).unwrap();

        Ok(())
    }
}
