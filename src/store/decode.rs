use crate::types::{ZNodeMeta, OBJ_TYPE_ZNODE, BlockId, BlockKind};
use crate::net_core::error::{NetError, NetResult};
use crate::codec::{
    tlv_iter,
    decode_l0_raw,
    decode_multi_recipe,
    decode_z_payload,
    decode_object_payload,
    ZPayload,
    ObjectPayload,
};
use crate::block::multi::MultiRecipe;

pub const MAGIC: [u8;4] = *b"QBLK";

fn u16_from(b: &[u8]) -> u16 { u16::from_be_bytes([b[0],b[1]]) }
fn u32_from(b: &[u8]) -> u32 { u32::from_be_bytes([b[0],b[1],b[2],b[3]]) }
fn u64_from(b: &[u8]) -> u64 { u64::from_be_bytes([b[0],b[1],b[2],b[3],b[4],b[5],b[6],b[7]]) }

/// Низкоуровневый разбор frame: header + raw payload.
pub fn decode_block_frame(buf: &[u8]) -> NetResult<(BlockKind, BlockId, [u8;32], Vec<u8>)> {
    if buf.len() < 4+1+1+2+4+32+8 {
        return Err(NetError::DecodeError);
    }
    if &buf[0..4] != MAGIC {
        return Err(NetError::DecodeError);
    }

    let kind = match buf[4] {
        0 => BlockKind::L0,
        1 => BlockKind::Multi,
        2 => BlockKind::Z,
        3 => BlockKind::Object,
        _ => return Err(NetError::DecodeError),
    };

    let _flags    = buf[5];
    let _reserved = u16_from(&buf[6..8]);
    let payload_len = u32_from(&buf[8..12]);

    let mut hash = [0u8;32];
    hash.copy_from_slice(&buf[12..44]);
    let id = u64_from(&buf[44..52]);

    let want = 52 + payload_len as usize;
    if buf.len() < want {
        return Err(NetError::DecodeError);
    }

    Ok((kind, id, hash, buf[52..want].to_vec()))
}

/// Типизированное содержимое блока (без id/hash/kind).
#[derive(Debug)]
pub enum BlockBody {
    L0(Vec<u8>),
    Multi(MultiRecipe),
    Z(ZPayload),
    Object(ObjectPayload),
}

/// Typed decode payload'а согласно BlockKind.
pub fn decode_l0_payload(payload: &[u8]) -> NetResult<Vec<u8>> {
    let tlvs = tlv_iter(payload)?;
    match decode_l0_raw(&tlvs) {
        Some(raw) => Ok(raw),
        None      => Err(NetError::DecodeError),
    }
}

pub fn decode_multi_payload(payload: &[u8]) -> NetResult<MultiRecipe> {
    let tlvs = tlv_iter(payload)?;
    match decode_multi_recipe(&tlvs) {
        Some(mr) => Ok(mr),
        None     => Err(NetError::DecodeError),
    }
}

pub fn decode_z_payload_from_bytes(payload: &[u8]) -> NetResult<ZPayload> {
    let tlvs = tlv_iter(payload)?;
    match decode_z_payload(&tlvs) {
        Some(z) => Ok(z),
        None    => Err(NetError::DecodeError),
    }
}

pub fn decode_object_payload_from_bytes(payload: &[u8]) -> NetResult<ObjectPayload> {
    let tlvs = tlv_iter(payload)?;
    match decode_object_payload(&tlvs) {
        Some(o) => Ok(o),
        None    => Err(NetError::DecodeError),
    }
}

/// Полное типизированное декодирование frame.
pub fn decode_block_typed(buf: &[u8]) -> NetResult<(BlockKind, BlockId, [u8;32], BlockBody)> {
    let (kind, id, hash, payload) = decode_block_frame(buf)?;

    let body = match kind {
        BlockKind::L0 => {
            let raw = decode_l0_payload(&payload)?;
            BlockBody::L0(raw)
        }
        BlockKind::Multi => {
            let mr = decode_multi_payload(&payload)?;
            BlockBody::Multi(mr)
        }
        BlockKind::Z => {
            let z = decode_z_payload_from_bytes(&payload)?;
            BlockBody::Z(z)
        }
        BlockKind::Object => {
            let o = decode_object_payload_from_bytes(&payload)?;
            BlockBody::Object(o)
        }
    };

    Ok((kind, id, hash, body))
}
