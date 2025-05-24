use std::path::{Path, PathBuf};
use std::fs::{self, Metadata};
use std::collections::HashSet;
use anyhow::{Result, anyhow, Context};
use blake3; // For fast cryptographic hashing
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathSecurityConfig {
    /// Allowed base directories
    pub allowed_dirs: Vec<PathBuf>,
    /// Maximum file size in bytes
    pub max_file_size: u64,
    /// Allowed file extensions
    pub allowed_extensions: HashSet<String>,
    /// Whether to allow symlinks
    pub allow_symlinks: bool,
    /// Whether to enforce file ownership checks
    pub check_ownership: bool,
}

impl Default for PathSecurityConfig {
    fn default() -> Self {
        Self {
            allowed_dirs: vec![],
            max_file_size: 10 * 1024 * 1024, // 10MB default
            allowed_extensions: HashSet::new(),
            allow_symlinks: false,
            check_ownership: true,
        }
    }
}

/// A secure path wrapper that enforces security restrictions
#[derive(Debug, Clone)]
pub struct SecurePath {
    path: PathBuf,
    config: PathSecurityConfig,
}

impl SecurePath {
    /// Create a new secure path with validation
    pub fn new<P: AsRef<Path>>(path: P, config: PathSecurityConfig) -> Result<Self> {
        let path_ref = path.as_ref();
        
        // Perform initial validation
        validate_path_basic(path_ref)?;
        
        // Canonicalize the path
        let canonical = path_ref.canonicalize()
            .context("Failed to canonicalize path")?;
            
        // Validate against security config
        validate_path_against_config(&canonical, &config)?;
        
        Ok(Self {
            path: canonical,
            config,
        })
    }

    /// Get the underlying path
    pub fn as_path(&self) -> &Path {
        &self.path
    }

    /// Check if the path exists
    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    /// Get file metadata with security checks
    pub fn metadata(&self) -> Result<Metadata> {
        let metadata = fs::metadata(&self.path)
            .context("Failed to get file metadata")?;
            
        validate_file_metadata(&metadata, &self.config)?;
        
        Ok(metadata)
    }

    /// Read file contents securely
    pub fn read(&self) -> Result<Vec<u8>> {
        // Validate file before reading
        let metadata = self.metadata()?;
        
        // Check file size
        if metadata.len() > self.config.max_file_size {
            return Err(anyhow!("File exceeds maximum allowed size"));
        }
        
        // Read file contents
        fs::read(&self.path)
            .context("Failed to read file contents")
    }

    /// Read file contents as string securely
    pub fn read_to_string(&self) -> Result<String> {
        let bytes = self.read()?;
        String::from_utf8(bytes)
            .context("File contents are not valid UTF-8")
    }

    /// Compute file hash
    pub fn compute_hash(&self) -> Result<String> {
        let contents = self.read()?;
        let hash = blake3::hash(&contents);
        Ok(hash.to_hex().to_string())
    }

    /// Validate file hash against expected value
    pub fn validate_hash(&self, expected: &str) -> Result<bool> {
        let actual = self.compute_hash()?;
        Ok(actual == expected)
    }
}

/// Basic path validation checks
fn validate_path_basic(path: &Path) -> Result<()> {
    // Check for directory traversal attempts
    let path_str = path.to_string_lossy();
    if path_str.contains("..") {
        return Err(anyhow!("Path contains directory traversal pattern"));
    }

    // Check path length
    if path_str.len() > 4096 {
        return Err(anyhow!("Path exceeds maximum length"));
    }

    // Check for suspicious characters
    let suspicious_chars = ['*', '?', '|', '<', '>', '`', '$', '&', ';'];
    if path_str.chars().any(|c| suspicious_chars.contains(&c)) {
        return Err(anyhow!("Path contains suspicious characters"));
    }

    Ok(())
}

/// Validate path against security configuration
fn validate_path_against_config(path: &Path, config: &PathSecurityConfig) -> Result<()> {
    // Check if path is within allowed directories
    if !config.allowed_dirs.is_empty() {
        let is_allowed = config.allowed_dirs.iter().any(|dir| {
            path.starts_with(dir)
        });
        
        if !is_allowed {
            return Err(anyhow!("Path is not in allowed directories"));
        }
    }

    // Check file extension
    if !config.allowed_extensions.is_empty() {
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                if !config.allowed_extensions.contains(ext_str) {
                    return Err(anyhow!("File extension not allowed"));
                }
            }
        }
    }

    // Check for symlinks if not allowed
    if !config.allow_symlinks {
        let metadata = fs::symlink_metadata(path)
            .context("Failed to get path metadata")?;
            
        if metadata.file_type().is_symlink() {
            return Err(anyhow!("Symbolic links are not allowed"));
        }
    }

    Ok(())
}

/// Validate file metadata against security requirements
fn validate_file_metadata(metadata: &Metadata, config: &PathSecurityConfig) -> Result<()> {
    // Check file size
    if metadata.len() > config.max_file_size {
        return Err(anyhow!("File exceeds maximum allowed size"));
    }

    // Check file ownership on Unix systems
    #[cfg(unix)]
    if config.check_ownership {
        use std::os::unix::fs::MetadataExt;
        
        let current_uid = unsafe { libc::getuid() };
        if metadata.uid() != current_uid {
            return Err(anyhow!("File is owned by different user"));
        }
    }

    Ok(())
}

/// Create a temporary file securely
pub fn create_secure_tempfile() -> Result<(PathBuf, fs::File)> {
    use tempfile::Builder;
    
    let file = Builder::new()
        .prefix("synx-secure-")
        .rand_bytes(16)
        .tempfile()?;
        
    let path = file.path().to_path_buf();
    let file = file.into_file();
    
    Ok((path, file))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::io::Write;

    #[test]
    fn test_secure_path_basic() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test content").unwrap();

        let config = PathSecurityConfig {
            allowed_dirs: vec![temp_dir.path().to_path_buf()],
            allowed_extensions: {
                let mut set = HashSet::new();
                set.insert("txt".to_string());
                set
            },
            ..Default::default()
        };

        let secure_path = SecurePath::new(&test_file, config).unwrap();
        assert!(secure_path.exists());
        assert_eq!(secure_path.read_to_string().unwrap(), "test content");
    }

    #[test]
    fn test_path_validation() {
        // Test directory traversal
        assert!(validate_path_basic(Path::new("../suspicious")).is_err());

        // Test suspicious characters
        assert!(validate_path_basic(Path::new("file*.txt")).is_err());
        assert!(validate_path_basic(Path::new("file;.txt")).is_err());
    }

    #[test]
    fn test_file_hash() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test content").unwrap();

        let config = PathSecurityConfig {
            allowed_dirs: vec![temp_dir.path().to_path_buf()],
            ..Default::default()
        };

        let secure_path = SecurePath::new(&test_file, config).unwrap();
        let hash = secure_path.compute_hash().unwrap();
        assert!(secure_path.validate_hash(&hash).unwrap());
    }

    #[test]
    fn test_secure_tempfile() {
        let (path, mut file) = create_secure_tempfile().unwrap();
        
        // Write some content
        writeln!(file, "test content").unwrap();
        
        // Verify content
        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content.trim(), "test content");
        
        // Clean up
        fs::remove_file(path).unwrap();
    }
}
