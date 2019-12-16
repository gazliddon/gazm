use std::net::{TcpListener};
use std::sync::mpsc;
use std::thread;

use crate::gdbstub::{ DebuggerHost, GdbRemote, Reply, Sigs};

#[derive(Debug, Clone, PartialEq)]
pub enum BreakPointTypes {
    Read,
    Write,
    Exec,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    Disconnected,
    Connected,
    DoBreak,
    ForcePc(u16),
    ReadRegisters,
    Resume,
    Step,
    WriteRegisters(Vec<u8>),
    Examine(u16),
    Write(u16, u8),
    Ack,
    IllegalInstruction,
    Halt(Sigs),
    BreakPoint(BreakPointTypes, u16),
    DeleteBreakPoint(BreakPointTypes, u16),
    SetReg(usize, u16),
    GetReg(usize),
}

struct DebuggerProxy {
    pub tx  : mpsc::Sender<Message>,
    pub rx  : mpsc::Receiver<Message>,
}

impl DebuggerProxy {

    pub fn new(tx : mpsc::Sender<Message>, rx : mpsc::Receiver<Message>) -> Self {
        Self { tx , rx } }

    pub fn send(&self, ev : Message) -> Message {
        let _ = self.tx.send(ev.clone());

        let received = self.rx.recv();

        if let Ok(msg) = received {
            msg
        } else {
            warn!("Error {:?}",received);
            panic!("msg system fucked")
        }
    }

    pub fn send_wait_ack(&self, ev : Message) {
        let msg = self.send(ev.clone());

        match msg {
            Message::Ack => (),
            _ => {
                warn!("sent {:?}", ev);
                panic!("Want ack got {:?}", msg)
            }
        };
    }

    pub fn ack(&self, _ev : Message) {
        let _ret = self.tx.send(Message::Ack);
    }

    pub fn serve(&mut self) {

        let listener = TcpListener::bind("127.0.0.1:6809").unwrap();

        loop {
            let mut gdb = GdbRemote::new(&listener);

            info!("GBD connected");

            self.send_wait_ack(Message::Connected);
            info!("Client confirmed connection");

            loop {
                if let Err(err)  = gdb.serve(self) {
                    warn!("Connection error {:?}", err);
                        break;
                }
            }

            self.send_wait_ack(Message::Disconnected);

            info!("Disconnected");
        }
    }
}

impl DebuggerHost for DebuggerProxy {

    fn do_break(&mut self) {
        self.send_wait_ack(Message::DoBreak);
    }

    fn read_registers(&self, response : &mut Reply)  {
        let reply = self.send(Message::ReadRegisters); 
        if let Message::WriteRegisters(data) = reply {
            for i in data {
                response.push_u8(i)
            }
        } else {
            panic!("examine: expected WriteRegisters got {:?}", reply)
        }
    }

    fn write_registers(&mut self, data : &[u8])  {
        self.send_wait_ack(Message::WriteRegisters(data.to_vec()));
    }

    fn examine(&self, addr : u16) -> u8 {
        let reply = self.send(Message::Examine(addr)) ;
        if let Message::Write(addr2, val) = reply {
            assert!(addr2 == addr);
            val
        } else {
            panic!("examine: expected Write got {:?}", reply)
        }
    }

    fn get_reg(&self, reg_num : usize) -> u16 {
        let reply = self.send(Message::GetReg(reg_num)) ;

        if let Message::SetReg(rnum, val) = reply {
            assert!(rnum == reg_num);
            val
        } else {
            panic!("get_reg: expected SetReg got {:?}", reply)
        }
    }

    fn set_reg(&self, reg_num : usize, val : u16) {
        self.send_wait_ack(Message::SetReg(reg_num, val));
    }

    fn write(&mut self, addr : u16, val : u8) {
        self.send_wait_ack(Message::Write(addr, val));
    }

    fn resume(&mut self) -> Sigs {
        let reply = self.send(Message::Resume);
        if let Message::Halt(sig) = reply {
            sig
        } else {
            panic!("resume: expected Halt got {:?}", reply)
        }
    }

    fn force_pc(&mut self, _pc : u16)  {
        self.send_wait_ack(Message::ForcePc(_pc));
    }

    fn set_step(&mut self) -> Sigs {
        self.send_wait_ack(Message::Step);
        Sigs::SIGTRAP
    }

    fn add_breakpoint(&mut self, addr : u16) {
        self.send_wait_ack(Message::BreakPoint(BreakPointTypes::Exec, addr));
    }

    fn add_write_watchpoint (&mut self, addr : u16) {
        self.send_wait_ack(Message::BreakPoint(BreakPointTypes::Write, addr));
    }

    fn add_read_watchpoint(&mut self, addr : u16) {
        self.send_wait_ack(Message::BreakPoint(BreakPointTypes::Read, addr));
    }

    fn del_breakpoint(&mut self, addr : u16) {
        self.send_wait_ack(Message::DeleteBreakPoint(BreakPointTypes::Exec, addr));
    }


    fn del_write_watchpoint(&mut self, addr : u16) {
        self.send_wait_ack(Message::DeleteBreakPoint(BreakPointTypes::Write, addr));
        unimplemented!("del_write_watchpoint");
    }

    fn del_read_watchpoint(&mut self, addr : u16) {
        self.send_wait_ack(Message::DeleteBreakPoint(BreakPointTypes::Read, addr));
    }
}

pub struct ThreadedGdb {
    pub rx  : mpsc::Receiver<Message>,
    pub tx  : mpsc::Sender<Message>,
}

////////////////////////////////////////////////////////////////////////////////
impl ThreadedGdb {

    pub fn new() -> ThreadedGdb {
        let (client_tx, server_rx) =  mpsc::channel(); 
        let (server_tx, client_rx) =  mpsc::channel(); 

        thread::spawn(move || {
            let mut dbg_proxy = DebuggerProxy::new(server_tx, server_rx);
            dbg_proxy.serve()
        });

        ThreadedGdb {
            tx : client_tx,
            rx : client_rx,
        }
    }

    pub fn reply(&mut self, msg : Message) {
        self.tx.send(msg).unwrap()
    }

    pub fn ack(&mut self) {
        self.reply(Message::Ack)
    }

    // poll from any messages from the debugger
    pub fn poll(&mut self) -> Option<Message> {
        match self.rx.try_recv() {
            Ok(msg) => Some(msg),
            Err(_) => None,
        }
    }
}

