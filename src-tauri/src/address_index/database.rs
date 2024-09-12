use super::types::Tx;

pub trait Database {
    async fn get_address_txs(&self) -> Vec<Tx>;
    async fn store_tx(&mut self, tx: Tx) -> crate::error::Result<()>;
}
