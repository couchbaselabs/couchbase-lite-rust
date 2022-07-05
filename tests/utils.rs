
extern crate couchbase_lite;

use couchbase_lite::Database;

use std::{ path::Path, sync::mpsc };

pub fn close_db(db_thread: std::thread::JoinHandle<()>, db_exec: DbQueryExecutor) {
    drop(db_exec);
    db_thread.join().expect("Couldn't join on the DB thread");
}

pub fn delete_db(db_path: &Path) {
    match Database::open(db_path.clone().to_str().unwrap(), None) {
        Ok(db) => {
            db.delete().unwrap();
        }
        Err(err) => {
            println!("Error: Initialiazion cause error: {}", err);
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
                Some(db)
            }
            Err(err) => {
                println!("Error: Initialiazion cause error: {}", err);
                None
            }
        };
        loop {
            match receiver.recv() {
                Ok(x) => x(&mut db),
                Err(err) => {
                    println!("Error: db_thread recv error: {}", err);
                    break;
                }
            }
        }
    });
    (join_handle, DbQueryExecutor { inner: sender })
}
