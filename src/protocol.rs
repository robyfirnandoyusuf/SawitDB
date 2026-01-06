use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Request {
    Ping { request_id: String },

    DbList { request_id: String },
    DbCreate { request_id: String, db: String },
    DbRename { request_id: String, db: String, new_db: String },
    DbDrop { request_id: String, db: String },

    Insert {
        request_id: String,
        db: String,
        collection: String,
        doc: Value,
    },
    Get {
        request_id: String,
        db: String,
        collection: String,
        id: String,
    },
    UpdateMerge {
        request_id: String,
        db: String,
        collection: String,
        id: String,
        patch: Value,
    },
    Delete {
        request_id: String,
        db: String,
        collection: String,
        id: String,
    },
    FindEq {
        request_id: String,
        db: String,
        collection: String,
        key: String,
        value: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub request_id: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Response {
    pub fn ok(request_id: String, result: Value) -> Self {
        Self { request_id, ok: true, result: Some(result), error: None }
    }
    pub fn err(request_id: String, msg: impl Into<String>) -> Self {
        Self { request_id, ok: false, result: None, error: Some(msg.into()) }
    }
}
