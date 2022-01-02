#![allow(unused_macros)]

use colored::*;
use nom::{AsChar, combinator::success};

pub trait Messageize {
    fn error(self) -> ColoredString;
    fn info(self) -> ColoredString;
    fn success(self) -> ColoredString;
}

impl<'a> Messageize for &'a str {
    fn error(self) -> ColoredString {
        self.red().bold()
    }

    fn info(self) -> ColoredString {
        self.blue().bold()
    }

    fn success(self) -> ColoredString {
        self.green().bold()
    }
}

pub struct Messages {
    indent : usize
}

impl Messages {
    pub fn new() -> Self {
        Self {
            indent : 0
        }
    }

    pub fn indent(&mut self) {
        self.indent += 1;
    }

    pub fn deindent(&mut self) {
        if self.indent > 0 {
            self.indent -= 1;
        }
    }

    fn get_indent(&self) -> usize {
        self.indent * 3
    }

    fn get_indent_str(&self) -> String {
        " ".repeat(self.get_indent())
    }

    pub fn info(&self,  m : &str) {
        println!("{}{}", self.get_indent_str(), m.blue());
    }

    pub fn error(&self,  m : &str) {
        println!("{}{}", self.get_indent_str(), m.red().bold());
    }

    
    pub fn warning(&self,  m : &str) {
        println!("{}{}", self.get_indent_str(), m.yellow().bold());
    }

    pub fn success(&self,  m : &str) {
        println!("{}{}", self.get_indent_str(), m.bold().green());
    }

    pub fn intertesting(&self, m : &str) {
        println!("{}{}", self.get_indent_str(), m.italic().bold().purple());
    }
    pub fn debug(&self, m : &str) {
        println!("{}{}", self.get_indent_str(), m.italic().yellow());
    }
}

use std::borrow::Borrow;
use std::sync::{Mutex, Once};
use std::time::Duration;
use std::{mem::MaybeUninit, thread};

struct SingletonReader {
    // Since we will be used in many threads, we need to protect
    // concurrent access
    inner: Mutex<u8>,
}

pub fn messages() -> &'static mut Messages {
    static mut SINGLETON: MaybeUninit<Messages> = MaybeUninit::uninit();
    static ONCE: Once = Once::new();

    unsafe {
        ONCE.call_once(|| {
            // Make it
            let singleton = Messages::new();
            SINGLETON.write(singleton);
        });

        // Now we give out a shared reference to the data, which is safe to use
        // concurrently.
        SINGLETON.assume_init_mut()
    }
}

