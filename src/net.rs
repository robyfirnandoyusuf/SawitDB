use anyhow::Result;
use bytes::{Buf, BufMut, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn read_frame(sock: &mut TcpStream, buf: &mut BytesMut) -> Result<Option<Vec<u8>>> {
    while buf.len() < 4 {
        let n = sock.read_buf(buf).await?;
        if n == 0 { return Ok(None); }
    }
    let mut header = &buf[..4];
    let len = header.get_u32() as usize;

    while buf.len() < 4 + len {
        let n = sock.read_buf(buf).await?;
        if n == 0 { return Ok(None); }
    }

    buf.advance(4);
    Ok(Some(buf.split_to(len).to_vec()))
}

pub async fn write_frame(sock: &mut TcpStream, payload: &[u8]) -> Result<()> {
    let mut out = BytesMut::with_capacity(4 + payload.len());
    out.put_u32(payload.len() as u32);
    out.extend_from_slice(payload);
    sock.write_all(&out).await?;
    Ok(())
}
