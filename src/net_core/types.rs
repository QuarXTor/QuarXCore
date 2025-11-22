use crate::types::{BlockId, ObjectId, CodecId, DictId, ClusterId};

/// NodeId: абстрактный идентификатор узла в сети.
pub type NodeId = u64;

/// Version tuple для сетевого протокола.
#[derive(Clone, Debug)]
pub struct ProtocolVersion {
    pub major: u16,
    pub minor: u16,
}

/// Идентификатор capability ключ/тип.
#[derive(Clone, Debug)]
pub enum CapabilityKind {
    Codecs,
    Dicts,
    Clusters,
}
