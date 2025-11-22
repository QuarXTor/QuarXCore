use crate::types::BlockRef;
use crate::codec::common::*;

/// Payload Object-блока (без id/hash).
#[derive(Clone, Debug)]
pub struct ObjectPayload {
    pub root:     BlockRef,
    pub obj_type: u32,
    pub meta:     Vec<u8>,
}

pub fn encode_object_payload(o: &ObjectPayload) -> Vec<u8> {
    let mut v = Vec::new();

    // 0x30 root ref
    let mut r = Vec::new();
    match o.root {
        BlockRef::L0(id)     => { r.push(0); r.extend_from_slice(&u64_encode(id)); }
        BlockRef::Multi(id)  => { r.push(1); r.extend_from_slice(&u64_encode(id)); }
        BlockRef::Z(id)      => { r.push(2); r.extend_from_slice(&u64_encode(id)); }
        BlockRef::Object(id) => { r.push(3); r.extend_from_slice(&u64_encode(id)); }
    }
    v.extend_from_slice(&tlv(0x30, &r));

    // 0x31 type
    v.extend_from_slice(&tlv(0x31, &u32_encode(o.obj_type)));

    // 0x32 meta
    v.extend_from_slice(&tlv(0x32, &o.meta));

    v
}

pub fn decode_object_payload(tlvs: &[(u8, Vec<u8>)]) -> Option<ObjectPayload> {
    let mut root = None;
    let mut t    = None;
    let mut meta = Vec::new();

    for (tag, val) in tlvs {
        match *tag {
            0x30 => {
                if val.len() != 1 + 8 { return None; }
                let kind = val[0];
                let id   = u64_decode(&val[1..]).ok()?;
                root = Some(match kind {
                    0 => BlockRef::L0(id),
                    1 => BlockRef::Multi(id),
                    2 => BlockRef::Z(id),
                    3 => BlockRef::Object(id),
                    _ => return None,
                });
            }
            0x31 => t = Some(u32_decode(val).ok()?),
            0x32 => meta = val.clone(),
            _ => {}
        }
    }

    Some(ObjectPayload {
        root: root?,
        obj_type: t?,
        meta,
    })
}
