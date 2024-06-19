use std::{
    ffi::OsString,
    fs::File,
    path::{Path, PathBuf},
};

use crate::args::ARGS;

pub struct TempFile {
    path: PathBuf,
    file: File,
}
impl TempFile {
    pub fn new<T: Into<OsString>>(file_name: T) -> Result<Self, std::io::Error> {
        let tmp_dir = ARGS.temp_dir();
        let tmp_file = tmp_dir.join(file_name.into());
        let file = File::create(&tmp_file)?;

        Ok(Self {
            path: tmp_file,
            file,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub const fn file(&self) -> &File {
        &self.file
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
