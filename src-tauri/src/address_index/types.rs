use serde::{Deserialize, Deserializer};

#[derive(Deserialize, Debug)]
pub struct Block {
    #[serde(rename = "tx")]
    pub txs: Vec<Tx>,
}
#[derive(Deserialize, Debug)]
pub struct Tx {
    pub txid: String,

    #[serde(deserialize_with = "concat_addresses")]
    #[serde(rename = "vout")]
    pub addresses: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct Vout {
    pub addresses: Vec<String>,
}

fn concat_addresses<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let vouts: Vec<Vout> = Vec::deserialize(deserializer)?;
    let mut addresses = vec![];
    for vout in vouts {
        addresses.extend(vout.addresses);
    }
    Ok(addresses)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_deserialization() -> Result<(), Box<dyn std::error::Error>> {
        let block: Block = serde_json::from_str(
            r#"
{
    "tx": [
        {
            "txid": "123",
            "vout": [
                {
                    "addresses": ["Address1"]
                },
                {
                    "addresses": ["Address2"]
                }
            ]
        },
        {
            "txid": "456",
            "vout": [
                {
                    "addresses": ["Address3"]
                },
                {
                    "addresses": ["Address4", "Address5"]
                }
            ]
        }
    ]
}
"#,
        )?;
        assert_eq!(block.txs.len(), 2);
        assert_eq!(block.txs[0].txid, "123");
        assert_eq!(block.txs[1].txid, "456");
        assert_eq!(block.txs[0].addresses, vec!["Address1", "Address2"]);
        assert_eq!(
            block.txs[1].addresses,
            vec!["Address3", "Address4", "Address5"]
        );
        Ok(())
    }
}
