use thiserror::Error;

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
    Unkown(#[from] wasmer_runtime::error::Error),
    #[error("Unable to call receive function in WASM module instance.")]
    Runtime(#[from] wasmer_runtime::error::RuntimeError),
    #[error("Actor reference not found.")]
    NoSuchActor,
}
