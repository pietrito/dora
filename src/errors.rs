pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub(crate) enum Error {
    FileOpen(std::io::Error),
    RewindFailed(std::io::Error),
    SeekFailed(std::io::Error),
    FlushError(std::io::Error),
    ReadError(std::io::Error),
    // TODO: Find out which error type is thrown by `String::parse()`
    ParseError,
}
