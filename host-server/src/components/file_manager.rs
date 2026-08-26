use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use tokio::fs;

use super::metadata::{GCodeMetadata, MetadataParser};

/// Manages root sandboxes for gcodes, config, and documentation files.
#[derive(Clone)]
pub struct FileManager {
    gcodes_root: PathBuf,
    config_root: PathBuf,
    parser: MetadataParser,
}

impl FileManager {
    pub fn new<P1: AsRef<Path>, P2: AsRef<Path>>(gcodes_root: P1, config_root: P2) -> Self {
        Self {
            gcodes_root: gcodes_root.as_ref().to_path_buf(),
            config_root: config_root.as_ref().to_path_buf(),
            parser: MetadataParser::new(),
        }
    }

    /// Resolve and sanitize requested relative path against base root.
    pub fn sanitize_path(&self, root: &Path, rel_path: &str) -> Result<PathBuf> {
        let path = Path::new(rel_path);
        let mut target = root.to_path_buf();

        for component in path.components() {
            match component {
                std::path::Component::Normal(c) => {
                    target.push(c);
                }
                std::path::Component::CurDir => {}
                std::path::Component::ParentDir
                | std::path::Component::RootDir
                | std::path::Component::Prefix(_) => {
                    return Err(anyhow!(
                        "Access denied: Path traversal outside sandbox detected"
                    ));
                }
            }
        }
        Ok(target)
    }

    pub fn get_root(&self, root_name: &str) -> Result<&PathBuf> {
        match root_name {
            "gcodes" => Ok(&self.gcodes_root),
            "config" => Ok(&self.config_root),
            _ => Err(anyhow!("Invalid root name: {}", root_name)),
        }
    }

    /// List files in the gcodes directory with metadata.
    pub async fn list_gcodes(&self) -> Result<Vec<GCodeMetadata>> {
        let mut results = Vec::new();
        if !self.gcodes_root.exists() {
            fs::create_dir_all(&self.gcodes_root).await?;
        }

        let mut read_dir = fs::read_dir(&self.gcodes_root).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "gcode" || ext == "gco" || ext == "g" {
                        if let Ok(meta) = self.parser.parse_file(&path) {
                            results.push(meta);
                        }
                    }
                }
            }
        }
        Ok(results)
    }

    /// Get metadata for a specific gcode file.
    pub async fn get_gcode_metadata(&self, filename: &str) -> Result<GCodeMetadata> {
        let target = self.sanitize_path(&self.gcodes_root, filename)?;
        if !target.exists() {
            return Err(anyhow!("File not found: {}", filename));
        }
        self.parser.parse_file(target)
    }

    /// Save/write a file within a sandbox root.
    pub async fn write_file(&self, root_name: &str, rel_path: &str, content: &[u8]) -> Result<()> {
        let root = self.get_root(root_name)?;
        let target = self.sanitize_path(root, rel_path)?;
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(target, content).await?;
        Ok(())
    }

    /// Delete a file within a sandbox root.
    pub async fn delete_file(&self, root_name: &str, rel_path: &str) -> Result<()> {
        let root = self.get_root(root_name)?;
        let target = self.sanitize_path(root, rel_path)?;
        if target.exists() {
            fs::remove_file(target).await?;
            Ok(())
        } else {
            Err(anyhow!("File not found: {}", rel_path))
        }
    }

    /// Move or rename a file within a sandbox root.
    pub async fn move_file(&self, root_name: &str, source_path: &str, dest_path: &str) -> Result<()> {
        let root = self.get_root(root_name)?;
        let src = self.sanitize_path(root, source_path)?;
        let dst = self.sanitize_path(root, dest_path)?;
        if !src.exists() {
            return Err(anyhow!("Source file not found: {}", source_path));
        }
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::rename(src, dst).await?;
        Ok(())
    }

    /// Copy a file within a sandbox root.
    pub async fn copy_file(&self, root_name: &str, source_path: &str, dest_path: &str) -> Result<()> {
        let root = self.get_root(root_name)?;
        let src = self.sanitize_path(root, source_path)?;
        let dst = self.sanitize_path(root, dest_path)?;
        if !src.exists() {
            return Err(anyhow!("Source file not found: {}", source_path));
        }
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::copy(src, dst).await?;
        Ok(())
    }
}
