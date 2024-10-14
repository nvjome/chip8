use std::fmt;

#[derive(Debug)]
pub enum CoreError {
    RomSizeError,
    ProgramCounterError,
}

impl std::error::Error for CoreError {}

impl std::fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CoreError::RomSizeError => write!(f, "invalid ROM size"),
            CoreError::ProgramCounterError => write!(f, "program counter out of bounds"),
        }
    }
}