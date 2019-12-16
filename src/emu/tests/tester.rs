use clap::{ArgMatches};

pub trait Tester {
    fn from_matches( args : &ArgMatches ) -> Self;
    fn run(&mut self);
    
}
