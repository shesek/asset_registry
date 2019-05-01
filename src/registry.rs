use std::sync::{Arc, Mutex};
use std::{fs, path, process::Command};

use bitcoin_hashes::hex::ToHex;
use elements::AssetId;

use crate::asset::Asset;
use crate::chain::ChainQuery;
use crate::errors::{OptionExt, Result, ResultExt};

// length of asset id prefix to use for sub-directory partitioning
// (in number of hex characters, not bytes)
const DIR_PARTITION_LEN: usize = 2;

#[derive(Debug)]
pub struct Registry {
    directory: path::PathBuf,
    chain: Option<ChainQuery>,
    hook_cmd: Option<String>,
    write_lock: Arc<Mutex<()>>,
}

impl Registry {
    pub fn new(
        directory: &path::Path,
        chain: Option<ChainQuery>,
        hook_cmd: Option<String>,
    ) -> Self {
        Registry {
            directory: directory.to_path_buf(),
            chain,
            hook_cmd,
            write_lock: Arc::new(Mutex::new(())),
        }
    }

    pub fn load(&self, asset_id: &AssetId) -> Result<Option<Asset>> {
        let name = format!("{}.json", asset_id.to_hex());
        let subdir = self.directory.join(&name[0..DIR_PARTITION_LEN]);
        let path = subdir.join(name);

        Ok(if path.exists() {
            Some(Asset::load(path)?)
        } else {
            None
        })
    }

    pub fn write(&self, asset: Asset) -> Result<()> {
        let _lock = self.write_lock.lock().unwrap();

        asset.verify(self.chain.as_ref())?;

        let name = format!("{}.json", asset.asset_id.to_hex());
        let subdir = self.directory.join(&name[0..DIR_PARTITION_LEN]);
        let path = subdir.join(name);

        if !subdir.exists() {
            fs::create_dir(&subdir)?;
        }

        fs::write(&path, serde_json::to_string(&asset)?)?;

        // XXX update index? or let the hook script take care of that?

        self.exec_hook(&asset.asset_id, &path)
            .context("hook script failed")?;

        Ok(())
    }

    pub fn exec_hook(&self, asset_id: &AssetId, asset_path: &path::Path) -> Result<()> {
        if let Some(cmd) = &self.hook_cmd {
            debug!("running hook: {}", cmd);

            let output = Command::new(cmd)
                .current_dir(&self.directory)
                .arg(asset_id.to_hex())
                .arg(asset_path.to_str().req()?)
                .output()?;
            debug!("hook output: {:?}", output);

            ensure!(output.status.success(), "hook script failed");
        }
        Ok(())
    }
}
