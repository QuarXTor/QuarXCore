use std::path::PathBuf;
use std::fs;

use smallvec::smallvec;

use quarxtor_core::store::file_store::FileBlockStore;
use quarxtor_core::store::blockstore::BlockStore;
use quarxtor_core::store::decode::BlockBody;
use quarxtor_core::types::{BlockRef};
use quarxtor_core::codec::{ZPayload, ObjectPayload};
use quarxtor_core::types::BlockKind;

#[test]
fn file_block_store_roundtrip() {
    let path: PathBuf = std::env::temp_dir().join("quarxtor_store_roundtrip.qblk");
    let _ = fs::remove_file(&path);

    // 1. Создаём store и пишем несколько блоков
    let mut store = FileBlockStore::open(path.clone()).expect("open store");

    // L0
    let id_l0 = store.put_l0(b"hello-l0").expect("put_l0");

    // Multi (Aggregate)
    let recipe = quarxtor_core::block::multi::MultiRecipe::Aggregate {
        blocks: smallvec![id_l0],
    };
    let id_multi = store.put_multi(&recipe).expect("put_multi");

    // Z-блок
    let z = ZPayload {
        first_l0: id_l0,
        last_l0:  id_l0,
        z_type:   1,
        meta:     Vec::new(),
    };
    let id_z = store.put_z(&z).expect("put_z");

    // Object
    let o = ObjectPayload {
        root:     BlockRef::Multi(id_multi),
        obj_type: 42,
        meta:     b"obj-meta".to_vec(),
    };
    let id_obj = store.put_object(&o).expect("put_object");

    // 2. Читаем назад типизированно
    let (k0, _h0, b0) = store.get_typed(id_l0).expect("get_typed l0");
    assert!(matches!(k0, BlockKind::L0));
    match b0 {
        BlockBody::L0(raw) => assert_eq!(raw, b"hello-l0"),
        _ => panic!("expected L0 body"),
    }

    let (k1, _h1, b1) = store.get_typed(id_multi).expect("get_typed multi");
    assert!(matches!(k1, BlockKind::Multi));
    match b1 {
        BlockBody::Multi(recipe2) => match recipe2 {
            quarxtor_core::block::multi::MultiRecipe::Aggregate { blocks } => {
                assert_eq!(blocks.len(), 1);
                assert_eq!(blocks[0], id_l0);
            }
            _ => panic!("expected Aggregate recipe"),
        },
        _ => panic!("expected Multi body"),
    }

    let (k2, _h2, b2) = store.get_typed(id_z).expect("get_typed z");
    assert!(matches!(k2, BlockKind::Z));
    match b2 {
        BlockBody::Z(z2) => {
            assert_eq!(z2.first_l0, id_l0);
            assert_eq!(z2.last_l0, id_l0);
            assert_eq!(z2.z_type, 1);
        }
        _ => panic!("expected Z body"),
    }

    let (k3, _h3, b3) = store.get_typed(id_obj).expect("get_typed obj");
    assert!(matches!(k3, BlockKind::Object));
    match b3 {
        BlockBody::Object(o2) => {
            match o2.root {
                BlockRef::Multi(mid) => assert_eq!(mid, id_multi),
                _ => panic!("expected root Multi(id_multi)"),
            }
            assert_eq!(o2.obj_type, 42);
            assert_eq!(o2.meta, b"obj-meta".to_vec());
        }
        _ => panic!("expected Object body"),
    }

    // 3. Закрываем и заново открываем, чтобы проверить индекс.
    drop(store);
    let store2 = FileBlockStore::open(path.clone()).expect("re-open store");

    let (k0b, _, _) = store2.get_typed(id_l0).expect("re-get l0");
    assert!(matches!(k0b, BlockKind::L0));

    let (k3b, _, _) = store2.get_typed(id_obj).expect("re-get obj");
    assert!(matches!(k3b, BlockKind::Object));

    let _ = fs::remove_file(&path);
}
