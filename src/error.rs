#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    SourceTargetMismatch,
    InvalidElement,
    PartialMap,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Self::SourceTargetMismatch => {
                write!(f, "cannot compose morphisms due to source/target mismatch.")
            }
            Self::InvalidElement => {
                write!(f, "element dimension does not match module dimension.")
            }
            Self::PartialMap => {
                write!(f, "element not found in map.")
            }
        }
    }
}

impl std::error::Error for Error {}
