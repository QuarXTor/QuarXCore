use std::path::PathBuf;
use std::fs;

use smallvec::smallvec;

use quarxtor_core::store::file_store::FileBlockStore;
use quarxtor_core::store::blockstore::BlockStore;
use quarxtor_core::codec::{ZPayload, ObjectPayload};
use quarxtor_core::types::{BlockRef, BlockKind};
use quarxtor_core::graph::ObjectGraph;

#[test]
fn object_graph_closure_simple_chain() {
    let path: PathBuf = std::env::temp_dir().join("quarxtor_object_graph.qblk");
    let _ = fs::remove_file(&path);

    let mut store = FileBlockStore::open(path.clone()).expect("open store");

    // L0
    let id_l0 = store.put_l0(b"hello-l0").expect("put_l0");

    // Multi -> L0
    let recipe = quarxtor_core::block::multi::MultiRecipe::Aggregate {
        blocks: smallvec![id_l0],
    };
    let id_multi = store.put_multi(&recipe).expect("put_multi");

    // Z -> [L0..L0]
    let z = ZPayload {
        first_l0: id_l0,
        last_l0:  id_l0,
        z_type:   1,
        meta:     Vec::new(),
    };
    let id_z = store.put_z(&z).expect("put_z");

    // Object -> Multi
    let o = ObjectPayload {
        root:     BlockRef::Multi(id_multi),
        obj_type: 7,
        meta:     Vec::new(),
    };
    let id_obj = store.put_object(&o).expect("put_object");

    // Graph
    let graph = ObjectGraph::new(&store);
    let closure = graph.compute_closure_from_object(id_obj).expect("closure");

    // Корень
    assert_eq!(closure.roots, vec![id_obj]);

    // В closure должны быть хотя бы все четыре блока
    for bid in [id_obj, id_multi, id_l0] {
        assert!(
            closure.blocks.contains(&bid),
            "closure missing block id {}",
            bid
        );
    }

    // Z-блок отдельно тоже должен давать замыкание на L0
    let closure_z = graph
        .compute_closure_from_block(id_z)
        .expect("closure_z");
    assert!(closure_z.blocks.contains(&id_z));
    assert!(closure_z.blocks.contains(&id_l0));

    // sanity: типы корректны
    let (k_obj, _, _) = store.get_typed(id_obj).expect("get obj typed");
    assert!(matches!(k_obj, BlockKind::Object));

    let (k_z, _, _) = store.get_typed(id_z).expect("get z typed");
    assert!(matches!(k_z, BlockKind::Z));

    let _ = fs::remove_file(&path);
}
