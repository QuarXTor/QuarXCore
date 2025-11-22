use crate::types::{BlockId, BlockKind};
use crate::codec::{
    ZPayload,
    ObjectPayload,
    encode_l0_raw,
    encode_multi_recipe,
    encode_z_payload,
    encode_object_payload,
};
use crate::block::multi::MultiRecipe;
use crate::store::decode::{decode_block_typed, BlockBody};
use crate::store::encode::encode_block;

use crate::net_core::error::NetError;

use blake3::hash;

/// Ошибки уровня хранилища.
#[derive(Debug)]
pub enum StoreError {
    Io(std::io::Error),
    Decode(NetError),
    OutOfRange(BlockId),
    Corrupt(String),
}

pub type StoreResult<T> = Result<T, StoreError>;

impl From<std::io::Error> for StoreError {
    fn from(e: std::io::Error) -> Self {
        StoreError::Io(e)
    }
}

impl From<NetError> for StoreError {
    fn from(e: NetError) -> Self {
        StoreError::Decode(e)
    }
}

/// Универсальный API хранилища блоков.
pub trait BlockStore {
    /// Записать L0-блок (сырые байты) и получить его BlockId.
    fn put_l0(&mut self, raw: &[u8]) -> StoreResult<BlockId>;

    /// Записать Multi-блок по рецепту.
    fn put_multi(&mut self, recipe: &MultiRecipe) -> StoreResult<BlockId>;

    /// Записать Z-блок (агрегация диапазона L0).
    fn put_z(&mut self, z: &ZPayload) -> StoreResult<BlockId>;

    /// Записать Object-блок (объектный DAG).
    fn put_object(&mut self, o: &ObjectPayload) -> StoreResult<BlockId>;

    /// Прочитать типизированный блок.
    fn get_typed(&self, id: BlockId) -> StoreResult<(BlockKind, [u8; 32], BlockBody)>;

    /// Прочитать raw frame как байты.
    fn get_frame(&self, id: BlockId) -> StoreResult<Vec<u8>>;
}

/// Вспомогательный хелпер: blake3(payload) -> [u8;32]
fn hash_payload(payload: &[u8]) -> [u8; 32] {
    let h = hash(payload);
    let mut out = [0u8; 32];
    out.copy_from_slice(h.as_bytes());
    out
}

/// Вспомогательный хелпер для типов, которые делают Frame на месте.
pub fn make_frame_l0(id: BlockId, raw: &[u8]) -> Vec<u8> {
    let payload = encode_l0_raw(raw);
    let h = hash_payload(&payload);
    encode_block(BlockKind::L0, id, &h, &payload)
}

pub fn make_frame_multi(id: BlockId, recipe: &MultiRecipe) -> Vec<u8> {
    let payload = encode_multi_recipe(recipe);
    let h = hash_payload(&payload);
    encode_block(BlockKind::Multi, id, &h, &payload)
}

pub fn make_frame_z(id: BlockId, z: &ZPayload) -> Vec<u8> {
    let payload = encode_z_payload(z);
    let h = hash_payload(&payload);
    encode_block(BlockKind::Z, id, &h, &payload)
}

pub fn make_frame_object(id: BlockId, o: &ObjectPayload) -> Vec<u8> {
    let payload = encode_object_payload(o);
    let h = hash_payload(&payload);
    encode_block(BlockKind::Object, id, &h, &payload)
}

/// Универсальный decode из raw frame в типизированное тело блока.
pub fn decode_frame_typed(buf: &[u8]) -> StoreResult<(BlockKind, BlockId, [u8; 32], BlockBody)> {
    let (kind, id, hash, body) = decode_block_typed(buf)?;
    Ok((kind, id, hash, body))
}
