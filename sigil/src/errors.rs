use std::{error::Error, fmt::Display, path::PathBuf};

#[derive(Debug)]
pub struct SyntaxError {
    line: Option<usize>,
    msg: String,
    file: Option<PathBuf>,
}

impl SyntaxError {
    pub fn new(l: usize, m: String, f: Option<PathBuf>) -> Self {
        SyntaxError {
            line: Some(l),
            msg: m,
            file: f,
        }
    }

    pub fn lnew<T: AsRef<str>>(m: T) -> Self {
        SyntaxError {
            line: None,
            msg: m.as_ref().to_string(),
            file: None,
        }
    }

    // This consumes the error
    pub fn print(self) {
        println!("\x1b[33m{}", "=".repeat(100));
        println!("\x1b[31m{}", self);
        println!("\x1b[33m{}\x1b[0m", "=".repeat(100));
    }
}

impl Display for SyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let file_spec = match (&self.file, self.line) {
            (None, None) => "",
            (None, Some(l)) => &format!(" (line {l})"),
            (Some(f), None) => &format!(" ({})", f.to_string_lossy()),
            (Some(f), Some(l)) => &format!(" ({}:{})", f.to_string_lossy(), l),
        };

        write!(f, "SyntaxError{}: {}", file_spec, self.msg)
    }
}

impl Error for SyntaxError {}

pub type SynErr = Result<(), SyntaxError>;
