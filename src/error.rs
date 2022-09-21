use crate::log_debug;

#[derive(thiserror::Error, Debug, Clone)]
pub enum CmdError {
    #[error("Error")] BaseError(String),
    #[error("Cannot run command")] OSProcessError(String),
    #[error("The command already exists")] DuplicateCmdError,
    #[error("CSV error")] CSVError(String),
    #[error("SQL error")] SQLError(String),
    #[error("Failed serializing/deserializing record")] CSVSerdeError(String),
}

impl std::convert::From<std::io::Error> for CmdError {
    fn from(err: std::io::Error) -> Self {
        log_debug!("{:?}", err.raw_os_error());
        CmdError::OSProcessError(err.to_string())
    }
}

impl std::convert::From<String> for CmdError {
    fn from(str: String) -> Self {
        CmdError::BaseError(str)
    }
}
impl std::convert::From<csv::Error> for CmdError {
    fn from(err: csv::Error) -> Self {
        match err.kind() {
            csv::ErrorKind::Serialize(err) => CmdError::CSVSerdeError(err.to_string()),
            csv::ErrorKind::Deserialize { pos: _, err } => CmdError::CSVSerdeError(err.to_string()),
            _ => CmdError::CSVError("There was an error with the CSV file".to_string()),
        }
    }
}

impl std::convert::From<sqlite::Error> for CmdError {
    fn from(err: sqlite::Error) -> Self {
        CmdError::SQLError(err.to_string())
    }
}