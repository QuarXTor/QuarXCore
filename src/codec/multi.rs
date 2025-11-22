use smallvec::SmallVec;

use crate::codec::common::*;
use crate::block::multi::{MultiRecipe, CodecRef, DictRef};
use crate::types::{BlockId, ClusterId, ObjectId};

/// MultiRecipe <-> TLV payload
///
/// Формат:
///   0x10: Aggregate
///       [ id_0:u64, id_1:u64, ... ]
///
///   0x11: CodecRecipe
///       codec_id:u64
///       codec_cluster:u64 (0 = None)
///       dict_flag:u8 (0/1)
///       if dict_flag==1:
///         dict_id:u64
///         dict_cluster:u64 (0=None)
///         dict_object_id:u64 (0=None)
///       recipe_id:u64
///       has_recipe_data:u8 (0/1)
///       if has_recipe_data==1:
///         recipe_len:u32
///         recipe_bytes[recipe_len]
///       has_blocks:u8 (0/1)
///       if has_blocks==1:
///         blocks_count:u32
///         blocks[blocks_count]*u64
///
///   0x12: Custom
///       kind_id:u32
///       payload:bytes
///
/// Всё это — только про recipe. id/hash/logical_len живут снаружи.

fn opt_cluster_to_u64(c: Option<ClusterId>) -> u64 {
    c.unwrap_or(0)
}
fn opt_obj_to_u64(o: Option<ObjectId>) -> u64 {
    o.unwrap_or(0)
}

pub fn encode_multi_recipe(recipe: &MultiRecipe) -> Vec<u8> {
    let mut v = Vec::new();

    match recipe {
        MultiRecipe::Aggregate { blocks } => {
            let mut buf = Vec::new();
            for id in blocks.iter() {
                buf.extend_from_slice(&u64_encode(*id));
            }
            v.extend_from_slice(&tlv(0x10, &buf));
        }

        MultiRecipe::CodecRecipe {
            codec,
            dict,
            recipe_id,
            recipe_data,
            blocks,
        } => {
            let mut buf = Vec::new();

            // codec
            buf.extend_from_slice(&u64_encode(codec.codec_id));
            buf.extend_from_slice(&u64_encode(opt_cluster_to_u64(codec.cluster)));

            // dict
            match dict {
                Some(d) => {
                    buf.push(1);
                    buf.extend_from_slice(&u64_encode(d.dict_id));
                    buf.extend_from_slice(&u64_encode(opt_cluster_to_u64(d.cluster)));
                    buf.extend_from_slice(&u64_encode(opt_obj_to_u64(d.object_id)));
                }
                None => buf.push(0),
            }

            // recipe_id
            buf.extend_from_slice(&u64_encode(*recipe_id));

            // recipe_data
            match recipe_data {
                Some(data) => {
                    buf.push(1);
                    buf.extend_from_slice(&u32_encode(data.len() as u32));
                    buf.extend_from_slice(data);
                }
                None => buf.push(0),
            }

            // blocks
            match blocks {
                Some(bv) => {
                    buf.push(1);
                    buf.extend_from_slice(&u32_encode(bv.len() as u32));
                    for id in bv.iter() {
                        buf.extend_from_slice(&u64_encode(*id));
                    }
                }
                None => buf.push(0),
            }

            v.extend_from_slice(&tlv(0x11, &buf));
        }

        MultiRecipe::Custom { kind_id, payload } => {
            let mut buf = Vec::new();
            buf.extend_from_slice(&u32_encode(*kind_id));
            buf.extend_from_slice(payload);
            v.extend_from_slice(&tlv(0x12, &buf));
        }
    }

    v
}

pub fn decode_multi_recipe(tlvs: &[(u8, Vec<u8>)]) -> Option<MultiRecipe> {
    for (tag, val) in tlvs {
        match *tag {
            0x10 => {
                // Aggregate
                if val.len() % 8 != 0 {
                    return None;
                }
                let mut ids: SmallVec<[BlockId; 8]> = SmallVec::new();
                for chunk in val.chunks(8) {
                    let id = u64_decode(chunk).ok()?;
                    ids.push(id);
                }
                return Some(MultiRecipe::Aggregate { blocks: ids });
            }

            0x11 => {
                let b = &val[..];
                let mut pos = 0usize;

                if b.len() < pos + 8 { return None; }
                let codec_id = u64_decode(&b[pos..pos+8]).ok()?; pos += 8;

                if b.len() < pos + 8 { return None; }
                let codec_cluster_raw = u64_decode(&b[pos..pos+8]).ok()?; pos += 8;
                let codec_cluster = if codec_cluster_raw == 0 { None } else { Some(codec_cluster_raw) };

                if b.len() < pos + 1 { return None; }
                let dict_flag = b[pos]; pos += 1;

                let mut dict: Option<DictRef> = None;
                if dict_flag == 1 {
                    if b.len() < pos + 8 + 8 + 8 { return None; }
                    let dict_id = u64_decode(&b[pos..pos+8]).ok()?; pos += 8;
                    let dict_cluster_raw = u64_decode(&b[pos..pos+8]).ok()?; pos += 8;
                    let dict_object_raw = u64_decode(&b[pos..pos+8]).ok()?; pos += 8;

                    let dict_cluster = if dict_cluster_raw == 0 { None } else { Some(dict_cluster_raw) };
                    let dict_object  = if dict_object_raw  == 0 { None } else { Some(dict_object_raw) };

                    dict = Some(DictRef {
                        dict_id,
                        cluster: dict_cluster,
                        object_id: dict_object,
                    });
                }

                if b.len() < pos + 8 { return None; }
                let recipe_id = u64_decode(&b[pos..pos+8]).ok()?; pos += 8;

                if b.len() < pos + 1 { return None; }
                let has_recipe_data = b[pos]; pos += 1;

                let mut recipe_data: Option<Vec<u8>> = None;
                if has_recipe_data == 1 {
                    if b.len() < pos + 4 { return None; }
                    let rd_len = u32_decode(&b[pos..pos+4]).ok()? as usize; pos += 4;
                    if b.len() < pos + rd_len { return None; }
                    recipe_data = Some(b[pos..pos+rd_len].to_vec());
                    pos += rd_len;
                }

                if b.len() < pos + 1 { return None; }
                let has_blocks = b[pos]; pos += 1;

                let mut blocks: Option<SmallVec<[BlockId; 8]>> = None;
                if has_blocks == 1 {
                    if b.len() < pos + 4 { return None; }
                    let count = u32_decode(&b[pos..pos+4]).ok()? as usize; pos += 4;

                    let mut ids: SmallVec<[BlockId; 8]> = SmallVec::new();
                    for _ in 0..count {
                        if b.len() < pos + 8 { return None; }
                        let id = u64_decode(&b[pos..pos+8]).ok()?; pos += 8;
                        ids.push(id);
                    }
                    blocks = Some(ids);
                }

                let codec = CodecRef {
                    codec_id,
                    cluster: codec_cluster,
                };

                return Some(MultiRecipe::CodecRecipe {
                    codec,
                    dict,
                    recipe_id,
                    recipe_data,
                    blocks,
                });
            }

            0x12 => {
                if val.len() < 4 {
                    return None;
                }
                let kind_id = u32_decode(&val[0..4]).ok()?;
                let payload = val[4..].to_vec();
                return Some(MultiRecipe::Custom {
                    kind_id,
                    payload,
                });
            }

            _ => {}
        }
    }

    None
}
