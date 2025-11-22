use smallvec::SmallVec;

use crate::types::{BlockId, ObjectId, CodecId, DictId, ClusterId};

/// Ссылка на кодек (абстрактная, кластер-ориентированная).
#[derive(Clone, Debug)]
pub struct CodecRef {
    pub codec_id:  CodecId,
    pub cluster:   Option<ClusterId>,
    // TODO: версия, ABI, OS-mask и т.п.
}

/// Ссылка на словарь внутри кодека.
#[derive(Clone, Debug)]
pub struct DictRef {
    pub dict_id:   DictId,
    pub cluster:   Option<ClusterId>,
    /// Опциональная ссылка на объект-граф, содержащий словарь как данные.
    pub object_id: Option<ObjectId>,
}

/// Recipe мультиблока (логический блок > L0).
#[derive(Clone, Debug)]
pub enum MultiRecipe {
    /// Просто агрегат из последовательности L0-блоков (B0..Bn).
    Aggregate {
        blocks: SmallVec<[BlockId; 8]>,
    },

    /// Рецепт, завязанный на кодек/словарь/кластер (в т.ч. пользовательский).
    CodecRecipe {
        codec:  CodecRef,
        dict:   Option<DictRef>,

        /// Идентификатор рецепта в пространстве (codec, dict).
        recipe_id: u64,

        /// Доп. данные рецепта (параметры, inline-структура, opaque payload).
        recipe_data: Option<Vec<u8>>,

        /// Опциональная фоллбек-связка с L0-блоками (B0..Bn).
        blocks: Option<SmallVec<[BlockId; 8]>>,
    },

    /// Зарезервировано под любые кастомные варианты (erasure codes, RAID-like, etc.).
    Custom {
        kind_id: u32,
        payload: Vec<u8>,
    },
}

/// MultiBlock: мультиблок / логический блок (например, 64K).
#[derive(Clone, Debug)]
pub struct MultiBlock {
    pub id:          BlockId,
    pub hash:        [u8; 32],  // хэш логического блока
    pub logical_len: u32,       // длина в байтах (64K и т.п.)
    pub recipe:      MultiRecipe,
}
