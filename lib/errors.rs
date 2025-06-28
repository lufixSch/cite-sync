use std::fmt;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct EnumConversionError;


impl fmt::Display for EnumConversionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "failed to convert enum")
    }
}

impl Error for EnumConversionError {}
