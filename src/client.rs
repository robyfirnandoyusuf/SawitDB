use anyhow::{anyhow, Result};
use bytes::BytesMut;
use tokio::net::TcpStream;
use url::Url;

use crate::{net, protocol::{Request, Response}};

#[derive(Clone, Debug)]
pub struct ConnInfo {
    pub host: String,
    pub port: u16,
    pub db: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

pub fn parse_sawit_uri(uri: &str) -> Result<ConnInfo> {
    let u = Url::parse(uri)?;
    if u.scheme() != "sawit" {
        return Err(anyhow!("scheme must be sawit://"));
    }
    let host = u.host_str().unwrap_or("127.0.0.1").to_string();
    let port = u.port().unwrap_or(27017);
    let db = {
        let p = u.path().trim_start_matches('/');
        if p.is_empty() { None } else { Some(p.to_string()) }
    };
    let username = if u.username().is_empty() { None } else { Some(u.username().to_string()) };
    let password = u.password().map(|s| s.to_string());

    Ok(ConnInfo { host, port, db, username, password })
}

pub struct Client {
    sock: TcpStream,
    buf: BytesMut,
}

impl Client {
    pub async fn connect(uri: &str) -> Result<Self> {
        let ci = parse_sawit_uri(uri)?;
        let addr = format!("{}:{}", ci.host, ci.port);
        let sock = TcpStream::connect(addr).await?;
        Ok(Self { sock, buf: BytesMut::with_capacity(8 * 1024) })
    }

    pub async fn call(&mut self, req: &Request) -> Result<Response> {
        let payload = serde_json::to_vec(req)?;
        net::write_frame(&mut self.sock, &payload).await?;

        let Some(resp_bytes) = net::read_frame(&mut self.sock, &mut self.buf).await? else {
            return Err(anyhow!("server closed connection"));
        };
        let resp: Response = serde_json::from_slice(&resp_bytes)?;
        Ok(resp)
    }
}
