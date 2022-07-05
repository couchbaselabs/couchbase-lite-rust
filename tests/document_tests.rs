
extern crate couchbase_lite;

use self::couchbase_lite::*;

pub mod utils;

#[test]
fn create_document() {
    utils::with_db(|_db| {
        let doc = Document::new_with_id("foo");
        assert_eq!(doc.id(), "foo");
        assert_eq!(doc.sequence(), 0);
        assert!(doc.properties());
        assert_eq!(doc.properties().count(), 0);
    });
}

#[test]
fn save_document() {
    utils::with_db(|db| {
        {
            let mut doc = Document::new_with_id("foo");
            let mut props = doc.mutable_properties();
            props.at("i").put_i64(1234);
            props.at("s").put_string("Hello World!");

            db.save_document(&mut doc, ConcurrencyControl::FailOnConflict).expect("save");
        }
        {
            let doc = db.get_document("foo").expect("reload document");
            let props = doc.properties();
            verbose!("Blah blah blah");
            info!("Interesting: {} = {}", 2+2, 4);
            warn!("This is a warning");
            error!("Oh no, props = {}", props);
            assert_eq!(props.to_json(), r#"{"i":1234,"s":"Hello World!"}"#);
        }
    });
}