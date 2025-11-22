use crate::types::{ZNodeMeta, OBJ_TYPE_ZNODE, BlockId, BlockKind};
use crate::codec::{
    encode_l0_raw,
    encode_multi_recipe,
    encode_z_payload,
    encode_object_payload,
    ZPayload,
    ObjectPayload,
};
use crate::block::multi::MultiRecipe;

/// Magic for block frame
pub const MAGIC: [u8;4] = *b"QBLK";

fn u16be(x: u16) -> [u8;2] { x.to_be_bytes() }
fn u32be(x: u32) -> [u8;4] { x.to_be_bytes() }
fn u64be(x: u64) -> [u8;8] { x.to_be_bytes() }

/// Финальная упаковка: header + payload
pub fn encode_block(kind: BlockKind, id: BlockId, hash: &[u8;32], payload: &[u8]) -> Vec<u8> {
    let mut v = Vec::new();
    v.extend_from_slice(&MAGIC);
    v.push(match kind {
        BlockKind::L0     => 0,
        BlockKind::Multi  => 1,
        BlockKind::Z      => 2,
        BlockKind::Object => 3,
    });
    v.push(0);                 // flags
    v.extend_from_slice(&u16be(0)); // reserved
    v.extend_from_slice(&u32be(payload.len() as u32));
    v.extend_from_slice(hash);
    v.extend_from_slice(&u64be(id));
    v.extend_from_slice(payload);
    v
}

/// ------------------------
/// Typed helpers (frames)
/// ------------------------

/// L0: сырые байты блока + id/hash -> полноценный frame.
pub fn encode_l0_frame(id: BlockId, hash: &[u8;32], raw: &[u8]) -> Vec<u8> {
    let payload = encode_l0_raw(raw);
    encode_block(BlockKind::L0, id, hash, &payload)
}

/// Multi: рецепт MultiRecipe -> frame.
pub fn encode_multi_frame(id: BlockId, hash: &[u8;32], recipe: &MultiRecipe) -> Vec<u8> {
    let payload = encode_multi_recipe(recipe);
    encode_block(BlockKind::Multi, id, hash, &payload)
}

/// Z: ZPayload (без id/hash) -> frame.
pub fn encode_z_frame(id: BlockId, hash: &[u8;32], z: &ZPayload) -> Vec<u8> {
    let payload = encode_z_payload(z);
    encode_block(BlockKind::Z, id, hash, &payload)
}

/// Object: ObjectPayload (без id/hash) -> frame.
pub fn encode_object_frame(id: BlockId, hash: &[u8;32], o: &ObjectPayload) -> Vec<u8> {
    let payload = encode_object_payload(o);
    encode_block(BlockKind::Object, id, hash, &payload)
}
