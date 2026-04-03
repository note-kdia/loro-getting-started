// https://loro.dev/docs/tutorial/get_started#save-and-load

use loro::*;

fn main() {
    let doc = LoroDoc::new();
    doc.get_text("text").insert(0, "Hello world!").unwrap();
    let data = doc.export(ExportMode::Snapshot).unwrap();

    let last_saved_version = doc.state_vv(); // JS: doc.version();
    doc.get_text("text").insert(0, "✨️").unwrap();

    // last_saved_version からの OpLog を取得する
    let update0 = doc
        .export(ExportMode::updates(&last_saved_version))
        .unwrap();
    let last_saved_version = doc.state_vv();

    doc.get_text("text").insert(0, "🧼").unwrap();
    let update1 = doc
        .export(ExportMode::updates(&last_saved_version))
        .unwrap();

    {
        println!("\n1. You can import the new snapshot and the updates");

        // import the snapshot
        let new_doc = LoroDoc::new();
        new_doc.import(&data).unwrap();
        println!("new_doc: {}", new_doc.get_deep_value().to_json());

        // import update0
        new_doc.import(&update0).unwrap();
        println!("update0: {}", new_doc.get_deep_value().to_json());

        // import update1
        new_doc.import(&update1).unwrap();
        println!("update1: {}", new_doc.get_deep_value().to_json());
    }

    {
        println!("\n2. You may also import them in a batch");

        // import the snapshot
        let new_doc = LoroDoc::new();
        new_doc.import_batch(&[data, update0, update1]).unwrap();
        println!("new_doc: {}", new_doc.get_deep_value().to_json());
    }
}
