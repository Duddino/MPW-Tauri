use super::types::Tx;

pub trait Database {
    async fn get_address_txids(&self, address: &str) -> crate::error::Result<Vec<String>>;
    async fn store_tx(&mut self, tx: Tx) -> crate::error::Result<()>;
}
