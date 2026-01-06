use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use serde_json::Value;
use uuid::Uuid;

use crate::client::Client;
use crate::protocol::Request;

#[derive(Parser)]
#[command(name="sawit", version, about="SawitDB-like CLI (custom TCP + sawit:// URI)")]
pub struct Cli {
    /// URI server, contoh: sawit://127.0.0.1:27017/toko
    #[arg(long, default_value="sawit://127.0.0.1:27017")]
    pub uri: String,

    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Subcommand)]
pub enum Cmd {
    Ping,

    DbList,
    DbCreate { db: String },
    DbRename { db: String, new_db: String },
    DbDrop { db: String },

    Insert { db: String, collection: String, doc: String },
    Get { db: String, collection: String, id: String },
    Update { db: String, collection: String, id: String, patch: String },
    Delete { db: String, collection: String, id: String },
    Find { db: String, collection: String, condition: String },
}

fn parse_obj(s: &str) -> Result<Value> {
    let v: Value = serde_json::from_str(s)?;
    if !v.is_object() { return Err(anyhow!("JSON must be object")); }
    Ok(v)
}

pub async fn run_cli(cli: Cli) -> Result<()> {
    let mut c = Client::connect(&cli.uri).await?;

    let req = match cli.cmd {
        Cmd::Ping => Request::Ping { request_id: Uuid::new_v4().to_string() },

        Cmd::DbList => Request::DbList { request_id: Uuid::new_v4().to_string() },
        Cmd::DbCreate { db } => Request::DbCreate { request_id: Uuid::new_v4().to_string(), db },
        Cmd::DbRename { db, new_db } => Request::DbRename { request_id: Uuid::new_v4().to_string(), db, new_db },
        Cmd::DbDrop { db } => Request::DbDrop { request_id: Uuid::new_v4().to_string(), db },

        Cmd::Insert { db, collection, doc } => Request::Insert {
            request_id: Uuid::new_v4().to_string(),
            db, collection, doc: parse_obj(&doc)?,
        },
        Cmd::Get { db, collection, id } => Request::Get {
            request_id: Uuid::new_v4().to_string(),
            db, collection, id,
        },
        Cmd::Update { db, collection, id, patch } => Request::UpdateMerge {
            request_id: Uuid::new_v4().to_string(),
            db, collection, id, patch: parse_obj(&patch)?,
        },
        Cmd::Delete { db, collection, id } => Request::Delete {
            request_id: Uuid::new_v4().to_string(),
            db, collection, id,
        },
        Cmd::Find { db, collection, condition } => {
            let (key, value) = condition.split_once('=').ok_or_else(|| anyhow!("condition must be key=value"))?;
            Request::FindEq {
                request_id: Uuid::new_v4().to_string(),
                db, collection,
                key: key.to_string(),
                value: value.to_string(),
            }
        }
    };

    let resp = c.call(&req).await?;
    if resp.ok {
        println!("{}", serde_json::to_string_pretty(&resp.result.unwrap_or(Value::Null))?);
        Ok(())
    } else {
        Err(anyhow!(resp.error.unwrap_or_else(|| "unknown error".into())))
    }
}
