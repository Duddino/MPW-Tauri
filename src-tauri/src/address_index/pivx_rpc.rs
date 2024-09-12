use super::block_source::BlockSource;
use super::types::Block;
use bitcoincore_rpc::{Auth, Client, RpcApi};
use futures::stream::{self, Stream};
use std::pin::Pin;

pub struct PIVXRpc {
    url: String,
    auth: Auth,
}

struct BlockIterator {
    client: Client,
    current_block: u64,
}

impl Iterator for BlockIterator {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
	let hash = self.client.get_block_hash(self.current_block).ok()?;
	let block = self.client.get_block(&hash).ok()?;
	self.current_block += 1;
	Some(block)
    }
}

impl PIVXRpc {
    pub fn new(url: &str, auth: Auth) -> Self {
        PIVXRpc {
            url: url.to_string(),
            auth,
        }
    }
    fn connect(&self) -> crate::error::Result<Client> {
        Ok(Client::new(&self.url, self.auth.clone())?)
    }
}

impl BlockSource for PIVXRpc {
    fn get_blocks(&mut self) -> crate::error::Result<Pin<Box<dyn Stream<Item = Block> + '_>>> {
	let block_iterator = BlockIterator {
	    client: self.connect()?,
	    current_block: 0,
	};

        Ok(Box::pin(stream::iter(block_iterator)))
    }
}
