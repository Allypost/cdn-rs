use std::{
    fs::File,
    io::{BufReader, Read, Write},
    path::{Path, PathBuf},
    time::{self, Instant},
};

use flate2::write::{DeflateEncoder, GzEncoder};
use strum::{EnumIter, IntoEnumIterator};
use tracing::{debug, trace};

use crate::tempfile::TempFile;

#[derive(Debug, Clone, Copy, EnumIter)]
pub enum Compression {
    Gzip,
    Deflate,
}

impl Compression {
    pub const fn file_ext(self) -> &'static str {
        match self {
            Self::Gzip => ".gz",
            Self::Deflate => ".zz",
        }
    }

    pub fn is_compressed(path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        Self::iter().any(|ct| path_str.ends_with(ct.file_ext()))
    }

    pub fn add_ext_to_file(self, path: &Path) -> PathBuf {
        let mut s = path.as_os_str().to_os_string();
        s.push(self.file_ext());

        PathBuf::from(s)
    }

    pub fn compress_file(self, path: &Path) -> anyhow::Result<()> {
        compress_file(path, self)
    }

    pub fn compress<R, W>(self, reader: &mut R, writer: &mut W) -> std::io::Result<()>
    where
        R: ?Sized + Read,
        W: ?Sized + Write,
    {
        match self {
            Self::Gzip => {
                let mut encoder = GzEncoder::new(writer, flate2::Compression::default());
                std::io::copy(reader, &mut encoder)?;
                encoder.try_finish()?;
            }

            Self::Deflate => {
                let mut encoder = DeflateEncoder::new(writer, flate2::Compression::default());
                std::io::copy(reader, &mut encoder)?;
                encoder.try_finish()?;
            }
        };

        Ok(())
    }
}

pub fn compress_file(path_original: &Path, compression: Compression) -> anyhow::Result<()> {
    if !path_original.exists() {
        anyhow::bail!("File does not exist: {:?}", path_original);
    }

    if Compression::is_compressed(path_original) {
        return Ok(());
    }

    let path_compressed = compression.add_ext_to_file(path_original);

    if path_compressed.exists() {
        return Ok(());
    }

    debug!(
        path=?path_original,
        typ=?compression,
        "Generating compressed file",
    );

    let tmp_file = TempFile::new(format!(
        "cdn-transcode-{c:?}-{ns:?}",
        c = compression,
        ns = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos(),
    ))?;

    let f = File::open(path_original)?;
    let mut input = BufReader::new(f);

    let start = Instant::now();
    compression.compress(&mut input, &mut tmp_file.file())?;
    debug!(
        path=?path_original,
        typ=?compression,
        "Compressed in {took:?}",
        took = start.elapsed(),
    );

    trace!(
        from=?tmp_file.path(),
        to=?path_compressed,
        "Copying compressed file to proper directory",
    );
    {
        let path_compressed_temp = {
            let mut s = path_compressed.as_os_str().to_os_string();
            s.push(".tmp");
            s
        };
        std::fs::copy(tmp_file.path(), &path_compressed_temp)?;
        std::fs::rename(&path_compressed_temp, &path_compressed)?;
    }
    trace!(
        from=?tmp_file.path(),
        to=?path_compressed,
        "Done",
    );

    Ok(())
}
