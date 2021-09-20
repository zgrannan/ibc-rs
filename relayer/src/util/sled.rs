#[cfg(feature="prusti")]
use prusti_contracts::*;

use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;

use crate::error::Error;

#[cfg(not(feature="prusti"))]
pub fn single<V>(prefix: impl Into<Vec<u8>>) -> SingleDb<V> {
    SingleDb::new(prefix)
}

#[cfg(feature="prusti")]
#[trusted]
pub fn single<T, V>(prefix: T) -> SingleDb<V> {
    todo!()
}

#[cfg(feature="prusti")]
#[trusted]
pub fn key_value<T, K, V>(prefix: T) -> KeyValueDb<K, V> {
    todo!()
}

#[cfg(not(feature="prusti"))]
pub fn key_value<K, V>(prefix: impl Into<Vec<u8>>) -> KeyValueDb<K, V> {
    KeyValueDb::new(prefix)
}

pub type SingleDb<V> = KeyValueDb<(), V>;

impl<V> SingleDb<V>
where
    V: Serialize + DeserializeOwned,
{
#[cfg_attr(feature="prusti_fast", trusted)]
    pub fn get(&self, db: &sled::Db) -> Result<Option<V>, Error> {
        self.fetch(db, &())
    }

#[cfg_attr(feature="prusti_fast", trusted)]
    pub fn set(&self, db: &sled::Db, value: &V) -> Result<(), Error> {
        self.insert(db, &(), value)
    }
}

#[cfg_attr(not(feature="prusti"), derive(Clone,Debug))]
pub struct KeyValueDb<K, V> {
    prefix: Vec<u8>,
    marker: PhantomData<(K, V)>,
}

impl<K, V> KeyValueDb<K, V> {

    #[cfg(feature="prusti")]
    #[trusted]
    pub fn new<T>(prefix: T) -> Self {
        todo!()
    }

    #[cfg(not(feature="prusti"))]
    pub fn new(prefix: impl Into<Vec<u8>>) -> Self {
        Self {
            prefix: prefix.into(),
            marker: PhantomData,
        }
    }
}

impl<K, V> KeyValueDb<K, V>
where
    K: Serialize,
    V: Serialize + DeserializeOwned,
{
#[cfg_attr(feature="prusti_fast", trusted)]
    fn prefixed_key(&self, mut key_bytes: Vec<u8>) -> Vec<u8> {
        let mut prefix_bytes = self.prefix.clone();
        prefix_bytes.append(&mut key_bytes);
        prefix_bytes
    }

#[cfg_attr(feature="prusti_fast", trusted)]
    pub fn fetch(&self, db: &sled::Db, key: &K) -> Result<Option<V>, Error> {
        let key_bytes = serde_cbor::to_vec(&key).map_err(Error::cbor)?;

        let prefixed_key_bytes = self.prefixed_key(key_bytes);

        let value_bytes = db.get(prefixed_key_bytes).map_err(Error::store)?;

        match value_bytes {
            Some(bytes) => {
                let value = serde_cbor::from_slice(&bytes).map_err(Error::cbor)?;
                Ok(value)
            }
            None => Ok(None),
        }
    }

    // pub fn has(&self, key: &K) -> Result<Option<V>, error::Error> {
    //     let key_bytes = serde_cbor::to_vec(&key).map_err(|e| error::Kind::Store.context(e))?;

    //     let exists = self
    //         .db
    //         .exists(key_bytes)
    //         .map_err(|e| error::Kind::Store.context(e))?;

    //     Ok(exists)
    // }

#[cfg_attr(feature="prusti_fast", trusted)]
    pub fn insert(&self, db: &sled::Db, key: &K, value: &V) -> Result<(), Error> {
        let key_bytes = serde_cbor::to_vec(&key).map_err(Error::cbor)?;

        let prefixed_key_bytes = self.prefixed_key(key_bytes);

        let value_bytes = serde_cbor::to_vec(&value).map_err(Error::cbor)?;

        db.insert(prefixed_key_bytes, value_bytes)
            .map(|_| ())
            .map_err(Error::store)?;

        Ok(())
    }
}
