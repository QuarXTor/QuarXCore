use crate::types::{BlockId};

/// L0Block: атомарный физический блок (обычно 8K).
#[derive(Clone, Debug)]
pub struct L0Block {
    pub id:   BlockId,
    pub hash: [u8; 32],
    pub size: u32,   // фактическая длина; обычно 8192
    pub tier: u8,    // уровень хранения (RAM/SSD/HDD/...)
    // TODO: segment, offset, flags, checksum-level, etc.
}
