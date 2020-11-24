use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("wrong cas")]
    Cas(Cas),

    #[error("repo is disconnected")]
    Disconnect(#[from] std::io::Error),
}

#[derive(Clone, Copy, Debug)]
pub struct Cas(u32);
impl fmt::Display for Cas {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

type Result<T> = std::result::Result<T, Error>;

pub trait AutoKey {
    fn key(&self) -> String;
}

struct User {
    id: u64,
    username: String,
}

impl AutoKey for User {
    fn key(&self) -> String {
        self.id.to_string()
    }
}

pub trait RepositoryExt<T>: Repository<T>
where
    T: AutoKey,
{
    fn create_doc(&mut self, doc: T) -> Result<Cas> {
        self.create(doc.key(), doc)
    }

    fn update_doc(&mut self, doc: T, cas: Cas) -> Result<Cas> {
        self.update(doc.key(), doc, cas)
    }

    fn delete_doc(&mut self, doc: T, cas: Cas) -> Result<Option<()>> {
        self.delete(doc.key(), cas)
    }
}
impl<D: AutoKey, T: Repository<D>> RepositoryExt<D> for T {}

struct Document<T> {
    cas: Cas,
    inner: T,
}

impl<T> Document<T> {
    fn cas(&self) -> Cas {
        self.cas
    }
}

pub trait Repository<T> {
    fn create(&mut self, key: String, doc: T) -> Result<Cas>;
    fn find<K: AsRef<str>>(&mut self, key: K) -> Result<Option<(T, Cas)>>;
    fn update<K: AsRef<str>>(&mut self, key: K, doc: T, cas: Cas) -> Result<Cas>;
    fn delete<K: AsRef<str>>(&mut self, key: K, cas: Cas) -> Result<Option<()>>;
}

use std::marker::PhantomData;

struct Logging<T, R: Repository<T>> {
    repo: R,
    _marker: PhantomData<T>,
}

impl<T, R: Repository<T>> Logging<T, R> {
    fn new(repo: R) -> Self {
        Self {
            repo,
            _marker: PhantomData,
        }
    }
}

impl<T, R: Repository<T>> Repository<T> for Logging<T, R>
where
    T: std::fmt::Debug,
{
    fn create(&mut self, key: String, doc: T) -> Result<Cas> {
        println!("CREATE {}: {:?}", key, doc);
        self.repo.create(key, doc)
    }

    fn find<K: AsRef<str>>(&mut self, key: K) -> Result<Option<(T, Cas)>> {
        println!("FIND {}", key.as_ref());
        self.repo.find(key)
    }

    fn update<K: AsRef<str>>(&mut self, key: K, doc: T, cas: Cas) -> Result<Cas> {
        println!("Update {} {}: {:?}", cas, key.as_ref(), doc);
        self.repo.update(key, doc, cas)
    }

    fn delete<K: AsRef<str>>(&mut self, key: K, cas: Cas) -> Result<Option<()>> {
        println!("DELETE {} {}", cas, key.as_ref());
        self.repo.delete(key, cas)
    }
}

pub mod inmem;
// pub mod fs
// pub mod s3
// pub mod redis
// pub mod couchbase
// pub mod cassandra
// pub mod  ...
