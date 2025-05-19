use std::fmt::Display;

#[derive(Debug)]
pub enum UserConnectedBaseError {
    InsertionFailure,
    BuildErrorNoDurationSet,
    BuildErrorNoTimeOutSet,
    Io(std::io::Error),
}

impl Display for UserConnectedBaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserConnectedBaseError::InsertionFailure => {
                write!(f, "InsertaionFailure")
            }
            UserConnectedBaseError::BuildErrorNoTimeOutSet => {
                write!(
                    f,
                    "User connected base : Build error  No Timeout callback set !"
                )
            }
            UserConnectedBaseError::BuildErrorNoDurationSet => {
                write!(f, "User connected base : Build error  No Duration set !")
            }

            UserConnectedBaseError::Io(e) => {
                write!(f, "Io error [{:?}]", e.to_string())
            }
        }
    }
}

impl From<std::io::Error> for UserConnectedBaseError {
    fn from(value: std::io::Error) -> Self {
        UserConnectedBaseError::Io(value)
    }
}
