#![allow(unused_macros)]

use colored::*;
use nom::{combinator::success, AsChar};

pub trait Messageize {
    fn error(self) -> ColoredString;
    fn info(self) -> ColoredString;
    fn success(self) -> ColoredString;
}
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Verbosity {
    NORMAL = 0,
    INFO = 1,
    INTERESTING = 2,
    DEBUG = 3,
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
    indent: usize,
    verbosity: Verbosity,
}

impl Messages {
    pub fn new() -> Self {
        Self {
            indent: 0,
            verbosity: Verbosity::NORMAL,
        }
    }

    pub fn set_verbosity(&mut self, verbosity : &Verbosity) {
        self.verbosity = *verbosity;
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

    pub fn info<S>(&self, m: S)
    where
        S: Into<String>,
    {
        if self.verbosity >= Verbosity::INFO {
            println!("{}{}", self.get_indent_str(), m.into().blue());
        }
    }

    pub fn status<S>(&self, m: S)
    where
        S: Into<String>,
    {
        println!("{}{}", self.get_indent_str(), m.into().bright_magenta());
    }

    pub fn error<S>(&self, m: S)
    where
        S: Into<String>,
    {
        println!("{}{}", self.get_indent_str(), m.into().red().bold());
    }

    pub fn warning<S>(&self, m: S)
    where
        S: Into<String>,
    {
        println!("{}{}", self.get_indent_str(), m.into().yellow().bold());
    }

    pub fn success<S>(&self, m: S)
    where
        S: Into<String>,
    {
        println!("{}{}", self.get_indent_str(), m.into().bold().green());
    }

    pub fn intertesting<S>(&self, m: S)
    where
        S: Into<String>,
    {
        if self.verbosity >= Verbosity::INTERESTING {
            println!(
                "{}{}",
                self.get_indent_str(),
                m.into().italic().bold().purple()
            );
        }
    }
    pub fn debug<S>(&self, m: S)
    where
        S: Into<String>,
    {
        if self.verbosity >= Verbosity::DEBUG {
            println!("{}{}", self.get_indent_str(), m.into().italic().yellow());
        }
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

////////////////////////////////////////////////////////////////////////////////
pub fn verbosity<F, Y>(verbosity : Verbosity, mut f: F) -> Y
where
    F: FnMut(&mut super::messages::Messages) -> Y,
{
    let x = super::messages::messages();
    let old_verbosity = x.verbosity;
    x.set_verbosity(&verbosity);

    let r = f(x);

    let x = super::messages::messages();
    x.set_verbosity(&old_verbosity);

    r
}

pub fn debug<F, Y>(text: &str, mut f: F) -> Y
where
    F: FnMut(&mut super::messages::Messages) -> Y,
{
    let x = super::messages::messages();
    x.debug(text);
    x.indent();
    let r = f(x);
    x.deindent();
    r
}

pub fn info<F, Y, S>(text: S, mut f: F) -> Y
where
    F: FnMut(&mut super::messages::Messages) -> Y,
    S: Into<String>
{
    let x = super::messages::messages();
    x.info(text.into());
    x.indent();
    let r = f(x);
    x.deindent();
    r
}

pub fn status<F, Y, S>(text: S, mut f: F) -> Y
where
    F: FnMut(&mut super::messages::Messages) -> Y,
    S: Into<String>
{
    let x = super::messages::messages();
    x.status(text.into());
    x.indent();
    let r = f(x);
    x.deindent();
    r
}
