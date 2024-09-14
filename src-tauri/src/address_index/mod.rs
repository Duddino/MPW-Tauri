pub mod block_source;
pub mod database;
pub mod pivx_rpc;
pub mod sql_lite;
pub mod types;

use block_source::BlockSource;
use database::Database;
use futures::StreamExt;

pub struct AddressIndex<D: Database, B: BlockSource> {
    database: D,
    block_source: B,
}

impl<D: Database + Send, B: BlockSource + Send> AddressIndex<D, B> {
    pub async fn sync(&mut self) -> crate::error::Result<()> {
        println!("Starting sync");
	let mut stream = self.block_source.get_blocks()?;
        while let Some(block) = stream.next().await {
            for tx in block.txs {
                self.database.store_tx(tx).await?;
            }
        }
        Ok(())
    }
    pub fn new(database: D, block_source: B) -> Self {
        Self {
            database,
            block_source,
        }
    }
}
