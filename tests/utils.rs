#![allow(unused_imports)]

extern crate couchbase_lite;

use couchbase_lite::{
    Database, DictKey, Query,
    Replicator, ReplicatorActivityLevel, ReplicatorStatus,
};
//use log::{error, trace};

use std::{
    path::Path,
    sync::{Arc, Mutex, mpsc},
    thread, time,
};

//#[derive(Debug)]
//pub enum TestResult { OK, KO }

pub fn close_db(db_thread: std::thread::JoinHandle<()>, db_exec: DbQueryExecutor) {
    //stop_replication(&db_exec);

    drop(db_exec);
    db_thread.join().expect("Couldn't join on the DB thread");
}

pub fn delete_db(db_path: &Path) {
    match Database::open(db_path.clone().to_str().unwrap(), None) {
        Ok(db) => {
            db.delete().unwrap();
        }
        Err(_err) => {
            //error!("Initialiazion cause error: {}", err);
        }
    };
}

type Job<T> = Box<dyn FnOnce(&mut Option<T>) + Send>;

#[derive(Clone)]
pub struct DbQueryExecutor {
    inner: mpsc::Sender<Job<Database>>,
}

impl DbQueryExecutor {
    pub fn spawn<F: FnOnce(&mut Option<Database>) + Send + 'static>(&self, job: F) {
        self.inner
            .send(Box::new(job))
            .expect("thread_pool::Executor::spawn failed");
    }
}

pub fn run_db_thread(db_path: &Path) -> (std::thread::JoinHandle<()>, DbQueryExecutor) {
    let (sender, receiver) = std::sync::mpsc::channel::<Job<Database>>();
    let db_path: std::path::PathBuf = db_path.into();
    let join_handle = std::thread::spawn(move || {
        let mut db = match Database::open(db_path.as_path().to_str().unwrap(), None) {
            Ok(db) => {
                /*println!("We read all messages after open:");
                print_all_messages(&db).expect("read from db failed");
                println!("read all messages after open done");*/
                Some(db)
            }
            Err(_err) => {
                //error!("Initialiazion cause error: {}", err);
                None
            }
        };
        loop {
            match receiver.recv() {
                Ok(x) => x(&mut db),
                Err(_err) => {
                    //trace!("db_thread: recv error: {}", err);
                    break;
                }
            }
        }
    });
    (join_handle, DbQueryExecutor { inner: sender })
}
