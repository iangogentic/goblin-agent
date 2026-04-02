use std::path::Path;

use goblin_app::FileRemoverInfra;

/// Low-level file remove service
///
/// Provides primitive file deletion operations without snapshot coordination.
/// Snapshot management should be handled at the service layer.
#[derive(Default)]
pub struct GoblinFileRemoveService;

impl GoblinFileRemoveService {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl FileRemoverInfra for GoblinFileRemoveService {
    async fn remove(&self, path: &Path) -> anyhow::Result<()> {
        Ok(goblin_fs::GoblinFS::remove_file(path).await?)
    }
}
