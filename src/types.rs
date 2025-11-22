
// ------------------------------------------------------------
// Z-node metadata (cheap-size / light analytics)
pub const OBJ_TYPE_ZNODE: u32 = 3;

#[derive(Debug, Clone, Copy)]
pub struct ZNodeMeta {
    pub size_bytes: u64,
    pub blocks: u32,
}
// ------------------------------------------------------------

pub type BlockId   = u64;
pub type ObjectId  = u64;
pub type CodecId   = u64;
pub type DictId    = u64;
pub type ClusterId = u64;

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BlockKind {
    L0     = 0,
    Multi  = 1,
    Z      = 2,
    Object = 3,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BlockRef {
    L0(BlockId),
    Multi(BlockId),
    Z(BlockId),
    Object(ObjectId),
}
