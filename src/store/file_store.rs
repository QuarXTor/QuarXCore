use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::Mutex;

use crate::types::{BlockId, BlockKind};
use crate::store::blockstore::{
    BlockStore, StoreError, StoreResult,
    make_frame_l0, make_frame_multi, make_frame_z, make_frame_object,
    decode_frame_typed,
};
use crate::store::decode::BlockBody;
use crate::store::encode::MAGIC;

use crate::codec::{ZPayload, ObjectPayload};
use crate::block::multi::MultiRecipe;

/// Простейшее reference-хранилище:
/// append-only файл + in-memory индекс id -> offset.
///
/// Формат frame см. в store::encode.
pub struct FileBlockStore {
    path: PathBuf,
    file: Mutex<File>,
    index: Vec<u64>, // offset для каждого BlockId
}

impl FileBlockStore {
    /// Открыть или создать файл-хранилище.
    /// При открытии производится сканирование файла и построение индекса.
    pub fn open(path: PathBuf) -> StoreResult<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;

        let mut store = Self {
            path,
            file: Mutex::new(file),
            index: Vec::new(),
        };

        store.rebuild_index()?;
        Ok(store)
    }

    fn rebuild_index(&mut self) -> StoreResult<()> {
        self.index.clear();

        let mut offset: u64 = 0;
        loop {
            let mut hdr = [0u8; 12];

            {
                let mut f = self
                    .file
                    .lock()
                    .map_err(|_| StoreError::Corrupt("file lock poisoned".into()))?;
                f.seek(SeekFrom::Start(offset))?;

                match f.read_exact(&mut hdr) {
                    Ok(()) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                        // дошли до конца файла
                        break;
                    }
                    Err(e) => return Err(StoreError::Io(e)),
                }
            }

            if &hdr[0..4] != &MAGIC {
                // считаем дальше файл поврежденным/не нашим; останавливаемся
                break;
            }

            let payload_len = u32_from(&hdr[8..12]) as u64;
            let frame_len = 12 + 32 + 8 + payload_len;

            self.index.push(offset);
            offset = offset
                .checked_add(frame_len)
                .ok_or_else(|| StoreError::Corrupt("offset overflow".into()))?;
        }

        Ok(())
    }

    fn next_id(&self) -> BlockId {
        self.index.len() as BlockId
    }

    fn read_frame_at(&self, offset: u64) -> StoreResult<Vec<u8>> {
        let mut hdr = [0u8; 12];

        {
            let mut f = self
                .file
                .lock()
                .map_err(|_| StoreError::Corrupt("file lock poisoned".into()))?;
            f.seek(SeekFrom::Start(offset))?;
            f.read_exact(&mut hdr)?;
        }

        if &hdr[0..4] != &MAGIC {
            return Err(StoreError::Corrupt("bad MAGIC".into()));
        }

        let payload_len = u32_from(&hdr[8..12]) as usize;
        let total_len = 12 + 32 + 8 + payload_len;
        let mut rest = vec![0u8; total_len - 12];

        {
            let mut f = self
                .file
                .lock()
                .map_err(|_| StoreError::Corrupt("file lock poisoned".into()))?;
            f.seek(SeekFrom::Start(offset + 12))?;
            f.read_exact(&mut rest)?;
        }

        let mut full = Vec::with_capacity(total_len);
        full.extend_from_slice(&hdr);
        full.extend_from_slice(&rest);
        Ok(full)
    }

    fn append_frame(&mut self, frame: &[u8]) -> StoreResult<BlockId> {
        let offset = {
            let mut f = self
                .file
                .lock()
                .map_err(|_| StoreError::Corrupt("file lock poisoned".into()))?;
            let offset = f.seek(SeekFrom::End(0))?;
            f.write_all(frame)?;
            f.flush()?;
            offset
        };

        let id = self.index.len() as BlockId;
        self.index.push(offset);
        Ok(id)
    }
}

fn u32_from(b: &[u8]) -> u32 {
    u32::from_be_bytes([b[0], b[1], b[2], b[3]])
}

impl BlockStore for FileBlockStore {
    fn put_l0(&mut self, raw: &[u8]) -> StoreResult<BlockId> {
        let id = self.next_id();
        let frame = make_frame_l0(id, raw);
        self.append_frame(&frame)
    }

    fn put_multi(&mut self, recipe: &MultiRecipe) -> StoreResult<BlockId> {
        let id = self.next_id();
        let frame = make_frame_multi(id, recipe);
        self.append_frame(&frame)
    }

    fn put_z(&mut self, z: &ZPayload) -> StoreResult<BlockId> {
        let id = self.next_id();
        let frame = make_frame_z(id, z);
        self.append_frame(&frame)
    }

    fn put_object(&mut self, o: &ObjectPayload) -> StoreResult<BlockId> {
        let id = self.next_id();
        let frame = make_frame_object(id, o);
        self.append_frame(&frame)
    }

    fn get_typed(&self, id: BlockId) -> StoreResult<(BlockKind, [u8; 32], BlockBody)> {
        if (id as usize) >= self.index.len() {
            return Err(StoreError::OutOfRange(id));
        }
        let offset = self.index[id as usize];
        let frame = self.read_frame_at(offset)?;
        let (kind, decoded_id, hash, body) = decode_frame_typed(&frame)?;

        if decoded_id != id {
            return Err(StoreError::Corrupt(format!(
                "id mismatch: requested {}, frame {}",
                id, decoded_id
            )));
        }

        Ok((kind, hash, body))
    }

    fn get_frame(&self, id: BlockId) -> StoreResult<Vec<u8>> {
        if (id as usize) >= self.index.len() {
            return Err(StoreError::OutOfRange(id));
        }
        let offset = self.index[id as usize];
        self.read_frame_at(offset)
    }
}
