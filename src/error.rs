#[derive(Debug)]
pub enum Error {
    UnexpectedCharacter,
    UnexpectedEndOfJson,
    UnparseableNumber,
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::UnexpectedCharacter => match other {
                Self::UnexpectedCharacter => true,
                _ => false,
            },
            Self::UnexpectedEndOfJson => match other {
                Self::UnexpectedEndOfJson => true,
                _ => false,
            },
            Self::UnparseableNumber => match other {
                Self::UnparseableNumber => true,
                _ => false,
            },
        }
    }
}
