use clap::{ArgMatches};
use crate::tests::tester;

impl tester::Tester for GregTest {
    fn from_matches( matches : &ArgMatches ) -> GregTest {
        GregTest {
            log_file : matches.value_of("LOG FILE").unwrap().to_string(),
        }
    }
    fn run(&mut self) {
    }
}

pub struct GregTest {
    pub log_file : String,
}


