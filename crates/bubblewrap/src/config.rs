use std::ffi::OsString;

#[derive(Clone, Debug)]
pub struct Config {
    pub(crate) program: OsString,
    pub(crate) args: Vec<OsString>,
}

impl Config {
    pub fn new(program: OsString) -> Self {
        Self {
            program,
            args: Vec::new(),
        }
    }

    pub fn args(&mut self, args: Vec<OsString>) -> &mut Self {
        self.args = args;
        self
    }
}
