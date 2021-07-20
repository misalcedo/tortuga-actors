use crate::TortugaError;
use futures::{AsyncRead, AsyncWrite};
use std::path::{Path, PathBuf};
use tokio::fs::{create_dir_all, remove_dir_all, File};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use walkdir::{DirEntry, WalkDir};

const TORTUGA_FILE_EXTENSION: &str = ".ta";
const WASM_FILE_EXTENSION: &str = "wasm";

/// A source file to be compiled.
pub struct CompilationSource {
    source: PathBuf,
    target: PathBuf,
}

impl CompilationSource {
    fn new<T: AsRef<Path>>(entry: &DirEntry, input: T) -> Result<CompilationSource, TortugaError> {
        let source = entry.path().to_path_buf();
        let target = source
            .strip_prefix(input)?
            .to_path_buf()
            .with_extension(WASM_FILE_EXTENSION);

        Ok(CompilationSource { source, target })
    }

    /// Open the source file to read for compilation.
    pub async fn source_file(&self) -> Result<impl AsyncRead + Unpin, TortugaError> {
        Ok(File::open(&self.source).await?.compat())
    }

    /// Create a file for writing the target of compiling this source.
    /// Creates all directories (including the parent) in the path that do not yet exist.
    pub async fn target_file<T: AsRef<Path>>(
        &self,
        parent_directory: T,
    ) -> Result<impl AsyncWrite + Unpin, TortugaError> {
        let filename = parent_directory.as_ref().join(&self.target);

        if let Some(parent) = filename.parent() {
            create_dir_all(parent).await?;
        }

        Ok(File::create(filename).await?.compat_write())
    }
}

/// Tests the entry has the Tortuga source file extension and is visible (i.e. not a dot-file).
fn is_tortuga_source(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.ends_with(TORTUGA_FILE_EXTENSION))
        .unwrap_or(false)
}

/// An iterator of the compilation sources in the given directory.
pub fn new_walker<T: AsRef<Path>>(sources: T) -> impl Iterator<Item = CompilationSource> {
    let sources = sources.as_ref().to_path_buf();

    WalkDir::new(&sources)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(is_tortuga_source)
        .filter_map(move |entry| CompilationSource::new(&entry, &sources).ok())
}

/// Cleans the given output directory.
pub async fn clean<T: AsRef<Path>>(output: T) -> Result<(), TortugaError> {
    match remove_dir_all(output).await {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}
