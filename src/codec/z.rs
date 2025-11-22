use crate::codec::common::*;
use crate::types::BlockId;

/// Структура только для payload Z-блока (без id/hash).
#[derive(Clone, Debug)]
pub struct ZPayload {
    pub first_l0: BlockId,
    pub last_l0:  BlockId,
    pub z_type:   u32,
    pub meta:     Vec<u8>,
}

pub fn encode_z_payload(z: &ZPayload) -> Vec<u8> {
    let mut v = Vec::new();
    v.extend_from_slice(&tlv(0x20, &u64_encode(z.first_l0)));
    v.extend_from_slice(&tlv(0x21, &u64_encode(z.last_l0)));
    v.extend_from_slice(&tlv(0x22, &u32_encode(z.z_type)));
    v.extend_from_slice(&tlv(0x23, &z.meta));
    v
}

pub fn decode_z_payload(tlvs: &[(u8, Vec<u8>)]) -> Option<ZPayload> {
    let mut first = None;
    let mut last  = None;
    let mut zt    = None;
    let mut meta  = Vec::new();

    for (tag, val) in tlvs {
        match *tag {
            0x20 => first = Some(u64_decode(val).ok()?),
            0x21 => last  = Some(u64_decode(val).ok()?),
            0x22 => zt    = Some(u32_decode(val).ok()?),
            0x23 => meta  = val.clone(),
            _ => {}
        }
    }

    Some(ZPayload {
        first_l0: first?,
        last_l0:  last?,
        z_type:   zt?,
        meta,
    })
}
