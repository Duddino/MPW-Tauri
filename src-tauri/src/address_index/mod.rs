mod block_source;
mod database;
mod pivx_rpc;
mod types;

use block_source::BlockSource;
use database::Database;
use futures::StreamExt;

pub struct AddressIndex<D: Database, B: BlockSource> {
    database: D,
    block_source: B,
}

impl<D: Database, B: BlockSource> AddressIndex<D, B> {
    pub async fn sync(&mut self) -> crate::error::Result<()> {
        while let Some(block) = self.block_source.get_blocks()?.next().await {
            for tx in block.txdata {
                self.database.store_tx(tx).await?;
            }
        }
        Ok(())
    }
}
