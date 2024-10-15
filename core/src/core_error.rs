/*
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
*/

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("invalid ROM size")]
    RomSizeError,
    #[error("program counter out of bounds\n index: {index}")]
    ProgramCounterError { index: u16 },
    #[error("index register out of bounds\n index: {index}")]
    IndexRegisterError { index: u16 },
    #[error("invalid opcode: {opcode}")]
    OpcodeError { opcode: u16 },
    #[error("cannot pop from empty stack")]
    StackEmptyError,
}