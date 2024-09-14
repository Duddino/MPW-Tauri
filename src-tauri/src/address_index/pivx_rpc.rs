use super::block_source::BlockSource;
use super::types::Block;
use futures::future::Pending;
use futures::stream::{self, Stream};
use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::HttpClient;
use jsonrpsee::rpc_params;
use reqwest::header::{HeaderMap, HeaderValue};
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
	println!("current block: {}", current_block);
        let hash: String = client
            .request("getblockhash", rpc_params![current_block])
            .await
            .unwrap();
        let block: Result<Block, _> = client.request("getblock", rpc_params![hash, 2]).await;
        if let Err(ref err) = &block {
            eprintln!("{}", err);
        }
        Some(block.ok()?)
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
            self.as_mut().current_block = self.current_block+ 1;
            let new_future = Box::pin(Self::get_next_block(
                self.client.clone(),
                self.current_block,
            ));
            self.current_future = Some(new_future);
            self.poll_next(cx)
        }
    }
}

impl PIVXRpc {
    pub async fn new(url: &str) -> crate::error::Result<Self> {
        let mut headers = HeaderMap::new();
	let credentials = format!("{}:{}", crate::RPC_USERNAME, crate::RPC_PASSWORD);
        headers.insert(
            "Authorization",
	    // TODO: remove unwrap
            HeaderValue::from_str(&format!("Basic {}", base64::encode(credentials))).unwrap()
        );
        Ok(PIVXRpc {
            client: HttpClient::builder().set_headers(headers).build(url)?,
        })
    }
}

impl BlockSource for PIVXRpc {
    fn get_blocks(
        &mut self,
    ) -> crate::error::Result<Pin<Box<dyn Stream<Item = Block> + Send + '_>>> {
        let block_stream = BlockStream::new(self.client.clone());

        Ok(Box::pin(block_stream))
    }
}
