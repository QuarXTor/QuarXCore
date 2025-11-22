use crate::types::{ObjectId, BlockRef};

/// Object: объект верхнего уровня (файл, снапшот, образ и т.п.).
#[derive(Clone, Debug)]
pub struct Object {
    pub id:       ObjectId,
    /// Корневой блок (может быть Multi/Z/дерево и т.п.).
    pub root:     BlockRef,
    /// Тип объекта (file, snapshot, vm-image, user-defined, etc.).
    pub obj_type: u32,
    /// Произвольные метаданные (имена, timestamps, ACL, user-defined payload).
    pub meta:     Vec<u8>,
}
