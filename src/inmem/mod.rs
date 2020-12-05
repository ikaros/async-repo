use super::{Cas, Error, Repository, Result};
use async_trait::async_trait;
use std::collections::HashMap;

pub struct Repo<T: Copy> {
    data: HashMap<String, CasDoc<T>>,
}

struct CasDoc<T> {
    cas: Cas,
    content: T,
}

impl<T> CasDoc<T> {
    pub fn new(content: T, cas: Cas) -> Self {
        Self { cas, content }
    }
}

// impl<T> From<CasDoc<T>> for (T, Cas) {
//     fn from(doc: CasDoc<T>) -> Self {
//         (doc.content, doc.cas)
//     }
// }

// impl <T> Into<T> for CasDoc<T> {
//     fn into(self) -> T {
//         self.content
//     }
// }

impl<T: Copy> Repo<T> {
    pub fn new() -> Self {
        Repo {
            data: HashMap::new(),
        }
    }
}

#[async_trait(?Send)]
impl<'a, D: Copy> Repository<D> for Repo<D> {
    async fn create(&mut self, key: String, doc: D) -> Result<Cas> {
        let cas = Cas(0);
        self.data.insert(key, CasDoc::new(doc, cas));
        Ok(Cas(0))
    }

    async fn find<T: AsRef<str>>(&mut self, key: T) -> Result<Option<(D, Cas)>> {
        match self.data.get(key.as_ref()) {
            Some(doc) => Ok(Some((doc.content, doc.cas.clone()))),
            None => Ok(None),
        }
    }

    async fn update<T: AsRef<str>>(&mut self, key: T, doc: D, cas: Cas) -> Result<Cas> {
        let cas = Cas(cas.0 + 1);
        self.data
            .insert(key.as_ref().to_owned(), CasDoc::new(doc, Cas(0)));
        Ok(cas)
    }

    async fn delete<T: AsRef<str>>(&mut self, key: T, _: Cas) -> Result<Option<()>> {
        match self.data.remove(key.as_ref()) {
            Some(_) => Ok(Some(())),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AutoKey;
    use crate::Logging;

    #[derive(Debug, Copy, Clone)]
    struct User {
        name: &'static str,
    }

    impl AutoKey for User {
        fn key(&self) -> String {
            self.name.to_owned()
        }
    }

    use tokio::test;

    #[tokio::test]
    async fn it_works() {
        let mut repo = Repo::new();
        let key = "mykey";

        let user_in = User { name: "Peter" };
        repo.create(key.to_string(), user_in)
            .await
            .expect("should not fail");
        let mut repo = Logging::new(repo);
        let (user_out, _) = repo.find(key).await.expect("not found").unwrap();
        assert_eq!(user_in.name, user_out.name);

        assert!(repo.find("asdf").await.is_ok());
        assert!(repo.find("asdf").await.expect("no error").is_none());
        // TODO: test fail with wrong cas
    }
}
