use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unable to decode UTF8 string.")]
    DecodeError(#[from] std::str::Utf8Error),
    #[error("Unable to parse WAT or WASM bytes.")]
    ParseError(#[from] wat::Error),
    #[error("Invalid WASM module.")]
    Invalid,
    #[error("Out-of-bounds memory access.")]
    PointerReference,
    #[error("Unable to compile the WASM module.")]
    Compile(#[from] wasmer_runtime::error::CompileError),
    #[error("Unable to instantiate the WASM module.")]
    Unknown(#[from] wasmer_runtime::error::Error),
    #[error("Unable to call receive function in WASM module instance.")]
    Runtime(#[from] wasmer_runtime::error::RuntimeError),
    #[error("Actor reference not found.")]
    NoSuchActor,
    #[error("Actor reference {0} is already registered.")]
    AlreadyRegistered(crate::reference::Reference),
    #[error("Unable to send the message to the specified actor.")]
    UnableToSend(#[from] crossbeam::channel::TrySendError<bytes::Bytes>),
    #[error("Unable to send the message to the specified actor via the broker.")]
    UnableToBroker(#[from] tokio::sync::mpsc::error::SendError<crate::message::Envelope>),
    #[error("Unable to receive a message from the specified actor.")]
    UnableToReceive(#[from] crossbeam::channel::TryRecvError),
}