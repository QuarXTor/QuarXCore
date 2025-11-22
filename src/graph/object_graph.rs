use std::collections::HashSet;

use crate::types::{BlockId, BlockKind, BlockRef};
use crate::codec::{ZPayload, ObjectPayload};
use crate::block::multi::MultiRecipe;
use crate::store::blockstore::{BlockStore, StoreResult, StoreError};
use crate::store::decode::BlockBody;

/// Результат замыкания графа относительно корня.
#[derive(Debug, Clone)]
pub struct GraphClosure {
    /// Корневые id (обычно один Object/Block).
    pub roots: Vec<BlockId>,
    /// Все достижимые блоки (включая корни) в порядке обхода.
    pub blocks: Vec<BlockId>,
}

/// Высокоуровневый обход ссылок (Multi/Z/Object) поверх BlockStore.
pub struct ObjectGraph<'a, S: BlockStore> {
    store: &'a S,
}

impl<'a, S: BlockStore> ObjectGraph<'a, S> {
    pub fn new(store: &'a S) -> Self {
        Self { store }
    }

    /// Построить замыкание, начиная с Object-блока (BlockKind::Object).
    pub fn compute_closure_from_object(&self, root_obj_id: BlockId) -> StoreResult<GraphClosure> {
        // sanity-check: корень действительно Object
        let (kind, _h, _body) = self.store.get_typed(root_obj_id)?;
        if !matches!(kind, BlockKind::Object) {
            return Err(StoreError::Corrupt(format!(
                "root {} is not Object (kind={:?})",
                root_obj_id, kind
            )));
        }
        self.compute_closure_from_block(root_obj_id)
    }

    /// Построить замыкание от произвольного блока (L0/Multi/Z/Object).
    pub fn compute_closure_from_block(&self, root_id: BlockId) -> StoreResult<GraphClosure> {
        let mut visited: HashSet<BlockId> = HashSet::new();
        let mut order: Vec<BlockId> = Vec::new();
        let mut stack: Vec<BlockId> = Vec::new();

        stack.push(root_id);

        while let Some(id) = stack.pop() {
            if !visited.insert(id) {
                continue;
            }

            let (kind, _hash, body) = self.store.get_typed(id)?;
            order.push(id);

            let children = children_from_body(kind, &body);
            for cid in children {
                if !visited.contains(&cid) {
                    stack.push(cid);
                }
            }
        }

        Ok(GraphClosure {
            roots: vec![root_id],
            blocks: order,
        })
    }
}

/// Извлечь дочерние BlockId из типизированного тела блока.
fn children_from_body(kind: BlockKind, body: &BlockBody) -> Vec<BlockId> {
    match (kind, body) {
        (BlockKind::L0, _) => Vec::new(),

        (BlockKind::Multi, BlockBody::Multi(recipe)) => children_from_multi(recipe),

        (BlockKind::Z, BlockBody::Z(zp)) => children_from_z(zp),

        (BlockKind::Object, BlockBody::Object(op)) => children_from_object(op),

        // несоответствие kind/body — считаем пустым (можно усилить позже)
        _ => Vec::new(),
    }
}

fn children_from_multi(recipe: &MultiRecipe) -> Vec<BlockId> {
    match recipe {
        MultiRecipe::Aggregate { blocks } => {
            blocks.iter().copied().collect()
        }
        MultiRecipe::CodecRecipe { blocks, .. } => {
            match blocks {
                Some(b) => b.iter().copied().collect(),
                None    => Vec::new(),
            }
        }
        // Для кастомных рецептов ядро ничего не знает про структуру ссылок.
        MultiRecipe::Custom { .. } => Vec::new(),
    }
}

fn children_from_z(zp: &ZPayload) -> Vec<BlockId> {
    if zp.last_l0 < zp.first_l0 {
        return Vec::new();
    }
    let count = zp
        .last_l0
        .saturating_sub(zp.first_l0)
        .saturating_add(1);
    let mut out = Vec::with_capacity(count as usize);
    let mut cur = zp.first_l0;
    while cur <= zp.last_l0 {
        out.push(cur);
        if cur == zp.last_l0 {
            break;
        }
        cur = cur.saturating_add(1);
    }
    out
}

fn children_from_object(op: &ObjectPayload) -> Vec<BlockId> {
    match op.root {
        BlockRef::L0(id)
        | BlockRef::Multi(id)
        | BlockRef::Z(id)
        | BlockRef::Object(id) => vec![id],
    }
}
