use std::path::Path;

use goblin_app::FileDirectoryInfra;

#[derive(Default)]
pub struct GoblinCreateDirsService;

#[async_trait::async_trait]
impl FileDirectoryInfra for GoblinCreateDirsService {
    async fn create_dirs(&self, path: &Path) -> anyhow::Result<()> {
        Ok(goblin_fs::GoblinFS::create_dir_all(path).await?)
    }
}
