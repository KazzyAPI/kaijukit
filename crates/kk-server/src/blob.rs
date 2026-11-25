//! Blob storage behind a trait so an object store can replace the local directory later.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub trait BlobStore: Send + Sync {
    fn put(&self, key: &str, data: &[u8]) -> io::Result<()>;
    fn get(&self, key: &str) -> io::Result<Vec<u8>>;
    fn delete(&self, key: &str) -> io::Result<()>;
}

pub struct LocalBlobs(PathBuf);

impl LocalBlobs {
    pub fn new(dir: impl AsRef<Path>) -> io::Result<Self> {
        fs::create_dir_all(dir.as_ref())?;
        Ok(Self(dir.as_ref().to_path_buf()))
    }

    /// Keys are hex digests; anything else could escape the directory.
    fn path(&self, key: &str) -> io::Result<PathBuf> {
        if key.is_empty() || !key.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid blob key",
            ));
        }
        Ok(self.0.join(&key[..2.min(key.len())]).join(key))
    }
}

impl BlobStore for LocalBlobs {
    fn put(&self, key: &str, data: &[u8]) -> io::Result<()> {
        let path = self.path(key)?;
        if path.exists() {
            return Ok(());
        }
        fs::create_dir_all(path.parent().unwrap_or(&self.0))?;
        let tmp = path.with_extension("tmp");
        fs::write(&tmp, data)?;
        fs::rename(tmp, path)
    }

    fn get(&self, key: &str) -> io::Result<Vec<u8>> {
        fs::read(self.path(key)?)
    }

    fn delete(&self, key: &str) -> io::Result<()> {
        match fs::remove_file(self.path(key)?) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_get_delete_and_key_validation() {
        let dir = std::env::temp_dir().join(format!("kk-blob-{}", std::process::id()));
        let b = LocalBlobs::new(&dir).unwrap();
        b.put("abcdef", b"hello").unwrap();
        b.put("abcdef", b"hello").unwrap();
        assert_eq!(b.get("abcdef").unwrap(), b"hello");
        b.delete("abcdef").unwrap();
        b.delete("abcdef").unwrap();
        assert!(b.get("abcdef").is_err());
        assert!(b.put("../x", b"").is_err());
        fs::remove_dir_all(dir).unwrap();
    }
}
