#[allow(dead_code)]
#[derive(thiserror::Error, Debug)]
pub enum ErrorThingy {
    #[error("Error: {0}")]
    Meow(String),
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
}
