// Couchbase Lite unit tests
//
// Copyright (c) 2020 Couchbase, Inc All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

extern crate couchbase_lite;
extern crate lazy_static;

use self::couchbase_lite::*;
use lazy_static::lazy_static;

mod utils;

use std::{
    path::Path,
    sync::{Arc, Mutex},
};

//////// TESTS:

#[test]
fn db_properties() {
    utils::with_db(|db| {
        assert_eq!(db.name(), utils::DB_NAME);
        assert_eq!(db.count(), 0);
    });
}

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

#[test]
fn query() {
    utils::with_db(|db| {
        utils::add_doc(db, "doc-1", 1, "one");
        utils::add_doc(db, "doc-2", 2, "two");
        utils::add_doc(db, "doc-3", 3, "three");

        let query = Query::new(db, QueryLanguage::N1QL, "select i, s from _ where i > 1 order by i").expect("create query");
        assert_eq!(query.column_count(), 2);
        assert_eq!(query.column_name(0), Some("i"));
        assert_eq!(query.column_name(1), Some("s"));

        // Step through the iterator manually:
        let results = query.execute().expect("execute");
        let mut row = (&results).next().unwrap(); //FIXME: Do something about the (&results). requirement
        let mut i = row.get(0).as_i64().unwrap();
        let mut s = row.get(1).as_string().unwrap();
        assert_eq!(i, 2);
        assert_eq!(s, "two");
        assert_eq!(row.as_dict().to_json(), r#"{"i":2,"s":"two"}"#);

        row = (&results).next().unwrap();
        i = row.get(0).as_i64().unwrap();
        s = row.get(1).as_string().unwrap();
        assert_eq!(i, 3);
        assert_eq!(s, "three");
        assert_eq!(row.as_dict().to_json(), r#"{"i":3,"s":"three"}"#);

        assert!((&results).next().is_none());

        // Now try a for...in loop:
        let mut n = 0;
        for row in &query.execute().expect("execute") {
            match n {
                0 => {
                    assert_eq!(row.as_array().to_json(), r#"[2,"two"]"#);
                    assert_eq!(row.as_dict().to_json(), r#"{"i":2,"s":"two"}"#);
                },
                1 => {
                    assert_eq!(row.as_array().to_json(), r#"[3,"three"]"#);
                    assert_eq!(row.as_dict().to_json(), r#"{"i":3,"s":"three"}"#);
                },
                _ => {panic!("Too many rows ({})", n);}
            }
            n += 1;

        }
        assert_eq!(n, 2);
    });
}

#[test]
fn in_transaction() {
    let path = Path::new("db");
    let (_db_thread, db_exec) = utils::run_db_thread(path);

    db_exec.spawn(move |db| {
        if let Some(db) = db.as_mut() {
            let result = db.in_transaction(transaction_callback);

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "document");

            let result = db.in_transaction(transaction_callback_error);

            assert!(result.is_err());

            assert!(db.get_document("document").is_ok());
            assert!(db.get_document("document_error").is_err());
        }
    });
}

fn transaction_callback(db: &mut Database) -> Result<String> {
    let mut doc = Document::new_with_id("document");
    db.save_document(&mut doc, ConcurrencyControl::LastWriteWins).unwrap();
    Ok("document".to_string())
}

fn transaction_callback_error(db: &mut Database) -> Result<String> {
    let mut doc = Document::new_with_id("document_error");
    db.save_document(&mut doc, ConcurrencyControl::LastWriteWins).unwrap();
    Err(couchbase_lite::Error::default())
}

lazy_static! {
    static ref BUFFER_NOTIFICATIONS: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    static ref DOCUMENT_DETECTED: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
}

#[test]
fn add_listener() {
    utils::set_static(&DOCUMENT_DETECTED, false);

    let path = Path::new("db");

    let (db_thread, db_exec) = utils::run_db_thread(path);

    db_exec.spawn(move |db| {
        if let Some(db) = db.as_mut() {
            let listener_token = db.add_listener(| _, doc_ids| {
                if doc_ids.first().unwrap() == "document" {
                    utils::set_static(&DOCUMENT_DETECTED, true);
                }
            });

            let mut doc = Document::new_with_id("document");
            db.save_document(&mut doc, ConcurrencyControl::LastWriteWins).unwrap();

            assert!(utils::check_static_with_wait(&DOCUMENT_DETECTED));

            drop(listener_token);
        } else {
            println!("Error: DB is NOT open");
        }
    });

    utils::close_db(db_thread, db_exec);
    utils::delete_db(path);
}

#[test]
fn buffer_notifications() {
    utils::set_static(&BUFFER_NOTIFICATIONS, false);
    utils::set_static(&DOCUMENT_DETECTED, false);

    let path = Path::new("db");

    let (db_thread, db_exec) = utils::run_db_thread(path);

    db_exec.spawn(move |db| {
        if let Some(db) = db.as_mut() {
            db.buffer_notifications(|_| {
                utils::set_static(&BUFFER_NOTIFICATIONS, true);
            });

            let listener_token = db.add_listener(| _, doc_ids| {
                if doc_ids.first().unwrap() == "document" {
                    utils::set_static(&DOCUMENT_DETECTED, true);
                }
            });

            let mut doc = Document::new_with_id("document");
            db.save_document(&mut doc, ConcurrencyControl::LastWriteWins).unwrap();

            assert!(!utils::check_static_with_wait(&DOCUMENT_DETECTED));
            assert!(utils::check_static_with_wait(&BUFFER_NOTIFICATIONS));

            db.send_notifications();

            assert!(utils::check_static_with_wait(&DOCUMENT_DETECTED));

            drop(listener_token);
        } else {
            println!("Error: DB is NOT open");
        }
    });

    utils::close_db(db_thread, db_exec);
    utils::delete_db(path);
}

/*
// This test doesn't and shouldn't compile -- it tests that the borrow-checker will correctly
// prevent Fleece data from being used after its document has been freed.
#[test]
fn document_borrow_check() {
    let mut db = Database::open(DB_NAME, None).expect("open db");
    let v : Value;
    {
        let doc = db.get_document("foo").expect("get doc");
        v = doc.properties().get("a");
    }
    println!("v = {:?}", v);
}
*/
