// https://loro.dev/docs/tutorial/get_started#container

use loro::*;

fn main() {
    let doc = LoroDoc::new();
    let list: LoroList = doc.get_list("list");

    list.insert(0, "A").unwrap();
    list.insert(1, "B").unwrap();
    list.insert(2, "C").unwrap();

    let map: LoroMap = doc.get_map("map");
    // map can only has string key
    map.insert("key", "value").unwrap(); // JS: map.set(key,value)
    println!("doc: {}", doc.get_deep_value().to_json());

    // delete 2 element at index 2
    list.delete(0, 2).unwrap();
    println!("doc: {}", doc.get_deep_value().to_json());

    // Insert a text container to the list
    let text = list.insert_container(0, LoroText::new()).unwrap();
    text.insert(0, "Hello").unwrap();
    text.insert(0, "Hi! ").unwrap();
    println!("doc: {}", doc.get_deep_value().to_json());

    // Insert a list container to the map
    let list2 = map.insert_container("test", LoroList::new()).unwrap();
    list2.insert(0, 1).unwrap();
    println!("doc: {}", doc.get_deep_value().to_json());
}
