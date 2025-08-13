use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub fn atomic_write<P: AsRef<Path>>(path: P, data: &[u8]) -> io::Result<()> {
    let path = path.as_ref();
    
    // Create temporary file path
    let mut temp_path = path.as_os_str().to_os_string();
    temp_path.push(".tmp");
    let temp_path = PathBuf::from(temp_path);
    
    // Write to temporary file
    let mut temp_file = File::create(&temp_path)?;
    temp_file.write_all(data)?;
    temp_file.sync_all()?;
    
    // Atomically rename temp file to target path
    fs::rename(temp_path, path)?;
    
    // Sync parent directory to ensure the rename is persisted
    if let Some(parent) = path.parent() {
        let _ = File::open(parent).and_then(|dir| dir.sync_all());
    }
    
    Ok(())
}