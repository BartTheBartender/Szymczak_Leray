#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    SourceTargetMismatch,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Self::SourceTargetMismatch => {
                write!(f, "cannot compose morphisms due to source/target mismatch.")
            }
        }
    }
}

impl std::error::Error for Error {}
