use crate::net_core::types::{NodeId, ProtocolVersion};
use crate::types::{BlockId, ObjectId};

/// Тип сетевого кадра (минимальный базис QuarXNet).
#[repr(u8)]
#[derive(Clone, Debug)]
pub enum FrameKind {
    Hello       = 1,
    Caps        = 2,
    GetBlocks   = 3,
    PushBlocks  = 4,
    GetObject   = 5,
    PushObject  = 6,
    Ping        = 7,
    Pong        = 8,
}

/// Заголовок сетевого кадра.
#[derive(Clone, Debug)]
pub struct FrameHeader {
    pub kind: FrameKind,
    pub flags: u8,
    pub length: u32, // длина payload
}

/// Универсальный сетевой кадр.
#[derive(Clone, Debug)]
pub struct Frame {
    pub header: FrameHeader,
    pub payload: Vec<u8>, // wire-format payload
}

/// Минимальные полезные payload-структуры.
#[derive(Clone, Debug)]
pub struct HelloPayload {
    pub node: NodeId,
    pub version: ProtocolVersion,
}

#[derive(Clone, Debug)]
pub struct GetBlocksPayload {
    pub ids: Vec<BlockId>,
}

#[derive(Clone, Debug)]
pub struct PushBlocksPayload {
    pub raw: Vec<u8>, // encoded L0/Multi/Z blocks
}

#[derive(Clone, Debug)]
pub struct GetObjectPayload {
    pub id: ObjectId,
}

#[derive(Clone, Debug)]
pub struct PushObjectPayload {
    pub raw: Vec<u8>, // encoded object+tree
}
