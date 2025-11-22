use crate::store::blockstore::BlockStore;
use std::sync::atomic::{AtomicU64, Ordering};

/// Snapshot статистики RAM-tier.
#[derive(Debug, Clone, Copy)]
pub struct RamStats {
    /// Лимит (байт), с которым был создан RamStore.
    /// 0        — слой по сути выключен;
    /// u64::MAX — "full/unlimited".
    pub limit_bytes: u64,

    /// Оценка занятой RAM кэшем (байт).
    pub used_bytes: u64,

    /// Кол-во блоков в кэше (по оценке).
    pub blocks: u64,

    pub hits: u64,
    pub misses: u64,
    pub inserts: u64,
    pub evictions: u64,
}

/// RamStore — будущий RAM-tier / кэш над любым BlockStore.
///
/// Сейчас:
///   - это точка расширения (тип присутствует во внешнем API),
///   - счётчики/лимит есть, но реального кэша ещё нет,
///   - BlockStore для RamStore не реализован, чтобы не ломать существующий код.
///
/// Дальше сюда добавится:
///   - внутренняя LRU / ARC-структура для блоков,
///   - impl BlockStore для прозрачного кэширования get/put.
#[derive(Debug)]
pub struct RamStore<S: BlockStore> {
    inner: S,
    /// Конфигурированный лимит RAM (байт).
    pub limit_bytes: u64,

    /// Счётчики и оценки (для будущего кэша).
    used_bytes: AtomicU64,
    blocks: AtomicU64,

    hits: AtomicU64,
    misses: AtomicU64,
    inserts: AtomicU64,
    evictions: AtomicU64,
}

impl<S: BlockStore> RamStore<S> {
    /// Создать RAM-обёртку над существующим BlockStore.
    ///
    /// Сейчас это просто прокладка с лимитом и счётчиками.
    pub fn new(inner: S, limit_bytes: u64) -> Self {
        Self {
            inner,
            limit_bytes,
            used_bytes: AtomicU64::new(0),
            blocks: AtomicU64::new(0),
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            inserts: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
        }
    }

    /// Доступ к базовому BlockStore (read-only).
    pub fn inner(&self) -> &S {
        &self.inner
    }

    /// Доступ к базовому BlockStore (mutable).
    pub fn inner_mut(&mut self) -> &mut S {
        &mut self.inner
    }

    /// Забрать внутренний Store.
    pub fn into_inner(self) -> S {
        self.inner
    }

    /// Текущий лимит RAM.
    pub fn limit(&self) -> u64 {
        self.limit_bytes
    }

    /// Включён ли RAM-tier логически (limit != 0).
    pub fn is_enabled(&self) -> bool {
        self.limit_bytes != 0
    }

    /// Режим "full/unlimited" (limit == u64::MAX).
    pub fn is_unlimited(&self) -> bool {
        self.limit_bytes == u64::MAX
    }

    /// Snapshot статистики RAM-tier.
    ///
    /// Пока все поля будут нулевыми (used_bytes/blocks/hits/...),
    /// т.к. кэширующая логика ещё не реализована.
    pub fn stats(&self) -> RamStats {
        RamStats {
            limit_bytes: self.limit_bytes,
            used_bytes: self.used_bytes.load(Ordering::Relaxed),
            blocks: self.blocks.load(Ordering::Relaxed),
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            inserts: self.inserts.load(Ordering::Relaxed),
            evictions: self.evictions.load(Ordering::Relaxed),
        }
    }

    /// Вспомогательные методы для будущего кэша.
    ///
    /// Сейчас не используются снаружи, но оставлены как контракт.
    pub(crate) fn add_used_bytes(&self, delta: i64) {
        if delta >= 0 {
            self.used_bytes
                .fetch_add(delta as u64, Ordering::Relaxed);
        } else {
            self.used_bytes
                .fetch_sub((-delta) as u64, Ordering::Relaxed);
        }
    }

    pub(crate) fn inc_blocks(&self, delta: i64) {
        if delta >= 0 {
            self.blocks.fetch_add(delta as u64, Ordering::Relaxed);
        } else {
            self.blocks.fetch_sub((-delta) as u64, Ordering::Relaxed);
        }
    }

    pub(crate) fn inc_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn inc_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn inc_insert(&self) {
        self.inserts.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn inc_eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }
}

/// Опциональное расширение для получения RAM-статистики.
///
/// Для RamStore возвращает Some(..), для обычных Store'ов — None.
pub trait RamBlockStoreExt {
    fn ram_stats(&self) -> Option<RamStats>;
}

impl<S: BlockStore> RamBlockStoreExt for RamStore<S> {
    fn ram_stats(&self) -> Option<RamStats> {
        Some(self.stats())
    }
}
