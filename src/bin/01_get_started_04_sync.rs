// https://loro.dev/docs/tutorial/get_started#sync

use loro::*;

fn main() {
    let doc_a = LoroDoc::new();
    let doc_b = LoroDoc::new();

    // node A inserts
    let list_a: LoroList = doc_a.get_list("list");
    list_a.insert(0, "A").unwrap();
    list_a.insert(1, "B").unwrap();
    list_a.insert(2, "C").unwrap();

    // B import the ops from A
    let data = doc_a.export(ExportMode::all_updates()).unwrap();
    // The data can be sent to B through the network
    doc_b.import(&data).unwrap();
    let last_oplog_vv = doc_b.oplog_vv();

    println!("doc_a: {}", doc_a.get_deep_value().to_json());
    println!("doc_b: {}", doc_b.get_deep_value().to_json());

    // node B deletes
    let list_b = doc_b.get_list("list");
    list_b.delete(1, 1).unwrap();

    let missing_ops = doc_b.export(ExportMode::updates(&last_oplog_vv)).unwrap();
    doc_a.import(&missing_ops).unwrap();

    println!("doc_a: {}", doc_a.get_deep_value().to_json());
    println!("doc_b: {}", doc_b.get_deep_value().to_json());
}
