use std::path::Path;

use anyhow::Result;
use goblin_app::FileInfoInfra;

pub struct GoblinFileMetaService;
#[async_trait::async_trait]
impl FileInfoInfra for GoblinFileMetaService {
    async fn is_file(&self, path: &Path) -> Result<bool> {
        Ok(goblin_fs::GoblinFS::is_file(path))
    }

    async fn is_binary(&self, path: &Path) -> Result<bool> {
        goblin_fs::GoblinFS::is_binary_file(path).await
    }

    async fn exists(&self, path: &Path) -> Result<bool> {
        Ok(goblin_fs::GoblinFS::exists(path))
    }

    async fn file_size(&self, path: &Path) -> Result<u64> {
        goblin_fs::GoblinFS::file_size(path).await
    }
}
