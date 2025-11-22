use crate::types::{CodecId, DictId, ClusterId};
use crate::net_core::types::CapabilityKind;

/// Capability объявления узла (поддерживаемые кодеки/словари/кластера).
#[derive(Clone, Debug)]
pub enum Capability {
    /// Перечень поддерживаемых CodecId.
    Codecs(Vec<CodecId>),

    /// Перечень поддерживаемых DictId.
    Dicts(Vec<DictId>),

    /// Список кластеров, которые этот узел может обслуживать.
    Clusters(Vec<ClusterId>),
}

impl Capability {
    pub fn kind(&self) -> CapabilityKind {
        match self {
            Capability::Codecs(_) => CapabilityKind::Codecs,
            Capability::Dicts(_) => CapabilityKind::Dicts,
            Capability::Clusters(_) => CapabilityKind::Clusters,
        }
    }
}
