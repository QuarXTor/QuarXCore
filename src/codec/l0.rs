use crate::codec::common::*;

/// L0: в payload храним просто сырые байты блока.
/// Здесь нет привязки к L0Block (id/hash/size/tier) — это уровень выше.

/// tag 0x01 = raw L0 bytes
pub fn encode_l0_raw(raw: &[u8]) -> Vec<u8> {
    tlv(0x01, raw)
}

pub fn decode_l0_raw(tlvs: &[(u8, Vec<u8>)]) -> Option<Vec<u8>> {
    for (tag, val) in tlvs {
        if *tag == 0x01 {
            return Some(val.clone());
        }
    }
    None
}
