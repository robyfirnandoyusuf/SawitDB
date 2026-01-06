use anyhow::Result;
use bytes::BytesMut;
use serde_json::json;
use tokio::net::{TcpListener, TcpStream};

use crate::{engine::Engine, net, protocol::{Request, Response}};

pub async fn serve(addr: &str, engine: Engine) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;
    loop {
        let (sock, _) = listener.accept().await?;
        let eng = engine.clone();
        tokio::spawn(async move {
            let _ = handle(sock, eng).await;
        });
    }
}

async fn handle(mut sock: TcpStream, engine: Engine) -> Result<()> {
    let mut buf = BytesMut::with_capacity(8 * 1024);

    while let Some(payload) = net::read_frame(&mut sock, &mut buf).await? {
        let req: Request = match serde_json::from_slice(&payload) {
            Ok(r) => r,
            Err(e) => {
                let resp = Response::err("unknown".into(), format!("bad request: {e}"));
                let out = serde_json::to_vec(&resp)?;
                net::write_frame(&mut sock, &out).await?;
                continue;
            }
        };

        let resp = dispatch(req, &engine).await;
        let out = serde_json::to_vec(&resp)?;
        net::write_frame(&mut sock, &out).await?;
    }

    Ok(())
}

async fn dispatch(req: Request, engine: &Engine) -> Response {
    use Request::*;

    match req {
        Ping { request_id } => Response::ok(request_id, json!({"pong": true})),

        DbList { request_id } => match engine.db_list() {
            Ok(dbs) => Response::ok(request_id, json!({"databases": dbs})),
            Err(e) => Response::err(request_id, e.to_string()),
        },

        DbCreate { request_id, db } => match engine.db_create(&db) {
            Ok(_) => Response::ok(request_id, json!({"created": db})),
            Err(e) => Response::err(request_id, e.to_string()),
        },

        DbRename { request_id, db, new_db } => match engine.db_rename(&db, &new_db) {
            Ok(_) => Response::ok(request_id, json!({"renamed_to": new_db})),
            Err(e) => Response::err(request_id, e.to_string()),
        },

        DbDrop { request_id, db } => match engine.db_drop(&db) {
            Ok(_) => Response::ok(request_id, json!({"dropped": db})),
            Err(e) => Response::err(request_id, e.to_string()),
        },

        Insert { request_id, db, collection, doc } => match engine.insert(&db, &collection, doc) {
            Ok(id) => Response::ok(request_id, json!({"id": id})),
            Err(e) => Response::err(request_id, e.to_string()),
        },

        Get { request_id, db, collection, id } => match engine.get(&db, &collection, &id) {
            Ok(doc) => Response::ok(request_id, json!({"doc": doc})),
            Err(e) => Response::err(request_id, e.to_string()),
        },

        UpdateMerge { request_id, db, collection, id, patch } => match engine.update_merge(&db, &collection, &id, patch) {
            Ok(updated) => Response::ok(request_id, json!({"updated": updated})),
            Err(e) => Response::err(request_id, e.to_string()),
        },

        Delete { request_id, db, collection, id } => match engine.delete(&db, &collection, &id) {
            Ok(deleted) => Response::ok(request_id, json!({"deleted": deleted})),
            Err(e) => Response::err(request_id, e.to_string()),
        },

        FindEq { request_id, db, collection, key, value } => match engine.find_eq(&db, &collection, &key, &value) {
            Ok(docs) => Response::ok(request_id, json!({"docs": docs})),
            Err(e) => Response::err(request_id, e.to_string()),
        },
    }
}
