use crate::net_core::error::{NetError, NetResult};

// utils
fn u32be(x: u32) -> [u8; 4] { x.to_be_bytes() }
fn u64be(x: u64) -> [u8; 8] { x.to_be_bytes() }

fn u32_from(b: &[u8]) -> u32 { u32::from_be_bytes([b[0], b[1], b[2], b[3]]) }
fn u64_from(b: &[u8]) -> u64 { u64::from_be_bytes([b[0],b[1],b[2],b[3],b[4],b[5],b[6],b[7]]) }

pub fn tlv(tag: u8, data: &[u8]) -> Vec<u8> {
    let mut v = Vec::new();
    v.push(tag);
    v.extend_from_slice(&u32be(data.len() as u32));
    v.extend_from_slice(data);
    v
}

pub fn tlv_iter(mut buf: &[u8]) -> NetResult<Vec<(u8, Vec<u8>)>> {
    let mut out = Vec::new();
    while !buf.is_empty() {
        if buf.len() < 1 + 4 {
            return Err(NetError::DecodeError);
        }
        let tag = buf[0];
        let len = u32_from(&buf[1..5]) as usize;
        buf = &buf[5..];
        if buf.len() < len {
            return Err(NetError::DecodeError);
        }
        let val = buf[..len].to_vec();
        buf = &buf[len..];
        out.push((tag, val));
    }
    Ok(out)
}

pub fn u64_encode(x: u64) -> Vec<u8> {
    u64be(x).to_vec()
}
pub fn u64_decode(b: &[u8]) -> NetResult<u64> {
    if b.len() != 8 { return Err(NetError::DecodeError); }
    Ok(u64_from(b))
}

pub fn u32_encode(x: u32) -> Vec<u8> {
    u32be(x).to_vec()
}
pub fn u32_decode(b: &[u8]) -> NetResult<u32> {
    if b.len() != 4 { return Err(NetError::DecodeError); }
    Ok(u32_from(b))
}
