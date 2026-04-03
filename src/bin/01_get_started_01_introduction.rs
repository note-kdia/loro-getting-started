// https://loro.dev/docs/tutorial/get_started#introduction

use loro::*;

fn main() {
    let doc_a = LoroDoc::new();
    let doc_b = LoroDoc::new();

    //...operation on doc_a and doc_b
    doc_a.get_text("t").insert(0, "Hello").unwrap();
    doc_b.get_text("t").insert(0, "World!").unwrap();

    // Assume doc_a and doc_b are two loro documents in two different devices
    let bytes_a = doc_a.export(ExportMode::all_updates()).unwrap();
    // send bytes to doc_b by any method
    doc_b.import(&bytes_a).unwrap();
    // doc_b is now updated with all the changes from doc_a

    let bytes_b = doc_b.export(ExportMode::all_updates()).unwrap();
    // send bytes to doc_a by any method
    doc_a.import(&bytes_b).unwrap();
    // doc_a and doc_b are now in sync, they have the same state

    println!("doc_a: {}", doc_a.get_text("t").to_string());
    println!("doc_b: {}", doc_b.get_text("t").to_string());

    assert_eq!(
        doc_a.get_text("t").to_string(),
        doc_b.get_text("t").to_string()
    )
}
