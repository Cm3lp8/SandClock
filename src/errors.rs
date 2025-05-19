use std::fmt::Display;

#[derive(Debug)]
pub enum SandClockError {
    InsertionFailure,
    BuildErrorNoDurationSet,
    BuildErrorNoTimeOutSet,
    Io(std::io::Error),
}

impl Display for SandClockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SandClockError::InsertionFailure => {
                write!(f, "InsertaionFailure")
            }
            SandClockError::BuildErrorNoTimeOutSet => {
                write!(
                    f,
                    "User connected base : Build error  No Timeout callback set !"
                )
            }
            SandClockError::BuildErrorNoDurationSet => {
                write!(f, "User connected base : Build error  No Duration set !")
            }

            SandClockError::Io(e) => {
                write!(f, "Io error [{:?}]", e.to_string())
            }
        }
    }
}

impl From<std::io::Error> for SandClockError {
    fn from(value: std::io::Error) -> Self {
        SandClockError::Io(value)
    }
}
