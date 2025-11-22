use crate::types::BlockId;

/// ZBlock: структурный блок (структурная зона поверх диапазона L0).
#[derive(Clone, Debug)]
pub struct ZBlock {
    pub id:       BlockId,
    pub hash:     [u8; 32],

    /// Диапазон L0-блоков, покрываемый этим Z-блоком (включительно).
    pub first_l0: BlockId,
    pub last_l0:  BlockId,

    /// Тип структурного блока (JSON, TAR, ELF-section, user-defined и т.п.).
    pub z_type:   u32,

    /// Opaque-метаданные, которые понимает только соответствующий анализатор/плагин.
    pub meta:     Vec<u8>,
}
