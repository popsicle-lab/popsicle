use artifact_system::document::Document;
use storage::{DocumentRow, MemoryDocumentStore};

#[test]
fn document_row_round_trips_body() {
    let mut d = Document::new("id-1", "prd", "Title");
    d.body = "body\nwith lines".into();
    let row = DocumentRow::from_document(&d, ".popsicle/artifacts/x.md");
    let back = row.to_document();
    assert_eq!(back.body, d.body);
    assert_eq!(back.id, d.id);
    assert_eq!(row.file_path, ".popsicle/artifacts/x.md");
}

#[test]
fn memory_store_insert_get() {
    let mut store = MemoryDocumentStore::new();
    let d = Document::new("a", "t", "T");
    let row = DocumentRow::from_document(&d, "a.md");
    store.insert(row).unwrap();
    assert_eq!(store.get("a").unwrap().title, "T");
}
