use super::block_source::BlockSource;
use super::types::Block;
use futures::future::Pending;
use futures::stream::{self, Stream};
use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::HttpClient;
use jsonrpsee::rpc_params;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct PIVXRpc {
    client: HttpClient,
}

struct BlockStream {
    client: HttpClient,
    current_block: u64,
    current_future: Option<Pin<Box<dyn Future<Output = Option<Block>> + Send>>>,
}

impl BlockStream {
    async fn get_next_block(client: HttpClient, current_block: u64) -> Option<Block> {
        let hash: String = client
            .request("getblockhash", rpc_params![current_block])
            .await
            .ok()?;
        let block: Block = client.request("getblock", rpc_params![hash]).await.ok()?;
        Some(block)
    }
    pub fn new(client: HttpClient) -> Self {
        Self {
            client,
            current_block: 0,
            current_future: None,
        }
    }
}

impl Stream for BlockStream {
    type Item = Block;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(ref mut future) = &mut self.current_future {
            let poll = Pin::as_mut(future).poll(cx);
            match poll {
                Poll::Ready(i) => {
                    self.current_future = None;
                    Poll::Ready(i)
                }
                Poll::Pending => Poll::Pending,
            }
        } else {
            self.current_block += 1;
            let new_future = Box::pin(Self::get_next_block(
                self.client.clone(),
                self.current_block,
            ));
            self.current_future.replace(new_future);
            self.poll_next(cx)
        }
    }
}

impl PIVXRpc {
    pub async fn new(url: &str) -> crate::error::Result<Self> {
        Ok(PIVXRpc {
            client: HttpClient::builder().build(url)?,
        })
    }
}

impl BlockSource for PIVXRpc {
    fn get_blocks(&mut self) -> crate::error::Result<Pin<Box<dyn Stream<Item = Block> + Send+ '_>>> {
        let block_stream = BlockStream::new(self.client.clone());

        Ok(Box::pin(block_stream))
    }
}
