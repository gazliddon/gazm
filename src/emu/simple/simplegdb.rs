use std::sync::mpsc;
use std::thread;

use std::net::{TcpListener};

use crate::gdbstub;

// pub enum Events {
//     HasDisconnected,
//     HasConnected,
//     Break,
//     Resume,
//     ReadRegisters,
//     Step,
//     Examine(u16),
//     Write(u16, u8),
//     SetRegisters,
//     GetRegisters(gdbstub::Reply),
//     ForcePc(u16),
// }
//

pub enum Events {
    DoBreak,
    ForcePc(u16),
    ReadRegisters,
    Resume,
    Step,
    WriteRegisters,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnState {
    Start,
    Waiting,
    Connected,
}

pub enum ConnEvent {
    HasDisconnected,
    HasConnected,
    HasNoEvent,
}

pub struct GdbConnection {
    state : ConnState,
    gdb : Option<gdbstub::GdbRemote>,
    tx : mpsc::Sender<gdbstub::GdbRemote>,
    rx : mpsc::Receiver<gdbstub::GdbRemote>,
}

impl GdbConnection {

    pub fn new() -> Self {
        use self::ConnState::*;

        let state = Start;
        let gdb = None;
        let (tx, rx) = mpsc::channel();

        Self {
            state, gdb, tx, rx
        }
    }

    // pub fn send_int(&mut self) {
    //     if let Some(ref mut remote) = self.gdb {
    //         let _ = remote.send_int();
    //     }
    // }

    pub fn update(&mut self, host : &mut dyn gdbstub::DebuggerHost) -> (ConnState, ConnEvent) {

        use self::ConnState::*;

        let mut event =  ConnEvent::HasNoEvent;

        match self.state {

            Start => {
                self.state = Waiting;

                let tx = self.tx.clone();

                thread::spawn(move || {
                    let listener = TcpListener::bind("127.0.0.1:6809").unwrap();
                    let rem = gdbstub::GdbRemote::new(&listener);
                    tx.send(rem).unwrap();
                });

                info!("Waiting for gdb connection")
            },

            Waiting => {
                let is_gdb = self.rx.try_recv();

                if is_gdb.is_ok() {
                    self.state = Connected;
                    event = ConnEvent::HasConnected;
                    self.gdb = Some(is_gdb.unwrap());
                    info!("gdb connected")
                }
            },

            Connected => {
                let mut ret = Err(());

                if let Some(ref mut remote) = self.gdb {
                    ret = remote.serve(host);
                }

                if ret.is_err() {
                    info!("gdb disconnected");
                    event = ConnEvent::HasDisconnected;
                    self.state = Start;
                }
            }

        }

        ( self.state, event )
    }
}

