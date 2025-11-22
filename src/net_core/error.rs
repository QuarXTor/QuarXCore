/// Ошибки низкоуровневого сетевого слоя.
#[derive(Debug)]
pub enum NetError {
    Io(std::io::Error),
    InvalidFrame,
    UnsupportedVersion,
    CapabilityMismatch,
    DecodeError,
    EncodeError,
}

pub type NetResult<T> = Result<T, NetError>;

impl From<std::io::Error> for NetError {
    fn from(e: std::io::Error) -> Self {
        NetError::Io(e)
    }
}
