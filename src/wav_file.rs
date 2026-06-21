use std::path::PathBuf;

use tempdir::TempDir;
use uuid::Uuid;

pub const URL_PATH_PREFIX: &'static str = "/wav_files";

pub fn name(uuid: Uuid) -> String {
    format!("{}.wav", uuid)
}

pub fn fs_path(temp_dir: &TempDir, uuid: Uuid) -> PathBuf {
    temp_dir.as_ref().join(&name(uuid))
}

pub fn url_path(uuid: Uuid) -> String {
    format!("{}/{}", URL_PATH_PREFIX, name(uuid))
}
