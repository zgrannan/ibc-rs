use alloc::collections::btree_map::BTreeMap as HashMap;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use crate::config::AddressType;
use bech32::{ToBase32, Variant};
use bip39::{Language, Mnemonic, Seed};
use bitcoin::{
    network::constants::Network,
    secp256k1::{Message, Secp256k1, SecretKey},
    util::bip32::{DerivationPath, ExtendedPrivKey, ExtendedPubKey},
};
use hdpath::StandardHDPath;
use ibc::core::ics24_host::identifier::ChainId;
use k256::ecdsa::{signature::Signer, Signature, SigningKey};
use ripemd160::Ripemd160;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tiny_keccak::{Hasher, Keccak};

use errors::Error;
pub use pub_key::EncodedPubKey;

pub mod errors;
mod pub_key;

pub type HDPath = StandardHDPath;

pub const KEYSTORE_DEFAULT_FOLDER: &str = ".hermes/keys/";
pub const KEYSTORE_DISK_BACKEND: &str = "keyring-test";
pub const KEYSTORE_FILE_EXTENSION: &str = "json";

// /!\ /!\ /!\ /!\ /!\ /!\ /!\ /!\ /!\ /!\ /!\ /!\ /!\ /!\ /!\
// WARNING: Changing this struct in backward incompatible way
//          will force users to re-import their keys.
// /!\ /!\ /!\ /!\ /!\ /!\ /!\ /!\ /!\ /!\ /!\ /!\ /!\ /!\ /!\
/// Key entry stores the Private Key and Public Key as well the address
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyEntry {
    /// Public key
    pub public_key: ExtendedPubKey,

    /// Private key
    pub private_key: ExtendedPrivKey,

    /// Account Bech32 format - TODO allow hrp
    pub account: String,

    /// Address
    pub address: Vec<u8>,
}

/// JSON key seed file
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyFile {
    pub name: String,
    pub r#type: String,
    pub address: String,
    pub pubkey: String,
    pub mnemonic: String,
}

impl KeyEntry {
}

pub trait KeyStore {
    fn add_key(&mut self, key_name: &str, key_entry: KeyEntry) -> Result<(), Error>;
    fn remove_key(&mut self, key_name: &str) -> Result<(), Error>;
    fn keys(&self) -> Result<Vec<(String, KeyEntry)>, Error>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Memory {
    account_prefix: String,
    keys: HashMap<String, KeyEntry>,
}

impl Memory {
    pub fn new(account_prefix: String) -> Self {
        Self {
            account_prefix,
            keys: HashMap::new(),
        }
    }
}

impl KeyStore for Memory {
    fn add_key(&mut self, key_name: &str, key_entry: KeyEntry) -> Result<(), Error> {
        if self.keys.contains_key(key_name) {
            Err(Error::key_already_exist())
        } else {
            self.keys.insert(key_name.to_string(), key_entry);

            Ok(())
        }
    }

    fn remove_key(&mut self, key_name: &str) -> Result<(), Error> {
        self.keys
            .remove(key_name)
            .ok_or_else(Error::key_not_found)?;

        Ok(())
    }

    fn keys(&self) -> Result<Vec<(String, KeyEntry)>, Error> {
        Ok(self
            .keys
            .iter()
            .map(|(n, k)| (n.to_string(), k.clone()))
            .collect())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Test {
    account_prefix: String,
    store: PathBuf,
}

impl Test {
    pub fn new(account_prefix: String, store: PathBuf) -> Self {
        Self {
            account_prefix,
            store,
        }
    }
}

impl KeyStore for Test {
    fn add_key(&mut self, key_name: &str, key_entry: KeyEntry) -> Result<(), Error> {
        let mut filename = self.store.join(key_name);
        filename.set_extension(KEYSTORE_FILE_EXTENSION);
        let file_path = filename.display().to_string();

        let file = File::create(filename).map_err(|e| {
            Error::key_file_io(file_path.clone(), "failed to create file".to_string(), e)
        })?;

        serde_json::to_writer_pretty(file, &key_entry)
            .map_err(|e| Error::key_file_encode(file_path, e))?;

        Ok(())
    }

    fn remove_key(&mut self, key_name: &str) -> Result<(), Error> {
        let mut filename = self.store.join(key_name);
        filename.set_extension(KEYSTORE_FILE_EXTENSION);

        fs::remove_file(filename.clone())
            .map_err(|e| Error::remove_io_fail(filename.display().to_string(), e))?;

        Ok(())
    }

    fn keys(&self) -> Result<Vec<(String, KeyEntry)>, Error> {
        unimplemented!()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Store {
    Memory,
    Test,
}

impl Default for Store {
    fn default() -> Self {
        Store::Test
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum KeyRing {
    Memory(Memory),
    Test(Test),
}

impl KeyRing {

    pub fn add_key(&mut self, key_name: &str, key_entry: KeyEntry) -> Result<(), Error> {
        match self {
            KeyRing::Memory(m) => m.add_key(key_name, key_entry),
            KeyRing::Test(d) => d.add_key(key_name, key_entry),
        }
    }

    pub fn remove_key(&mut self, key_name: &str) -> Result<(), Error> {
        match self {
            KeyRing::Memory(m) => m.remove_key(key_name),
            KeyRing::Test(d) => d.remove_key(key_name),
        }
    }

    pub fn keys(&self) -> Result<Vec<(String, KeyEntry)>, Error> {
        match self {
            KeyRing::Memory(m) => m.keys(),
            KeyRing::Test(d) => d.keys(),
        }
    }

    /// Sign a message
    pub fn sign_msg(
        &self,
        key_name: &str,
        msg: Vec<u8>,
        address_type: &AddressType,
    ) -> Result<Vec<u8>, Error> {
        unimplemented!()
    }

    pub fn account_prefix(&self) -> &str {
        match self {
            KeyRing::Memory(m) => &m.account_prefix,
            KeyRing::Test(d) => &d.account_prefix,
        }
    }
}

/// Sign a message
pub fn sign_message(
    key: &KeyEntry,
    msg: Vec<u8>,
    address_type: &AddressType,
) -> Result<Vec<u8>, Error> {
    let private_key_bytes = key.private_key.to_priv().to_bytes();
    match address_type {
        AddressType::Ethermint { ref pk_type } if pk_type.ends_with(".ethsecp256k1.PubKey") => {
            let hash = keccak256_hash(msg.as_slice());
            let s = Secp256k1::signing_only();
            // SAFETY: hash is 32 bytes, as expected in `Message::from_slice` -- see `keccak256_hash`, hence `unwrap`
            let sign_msg = Message::from_slice(hash.as_slice()).unwrap();
            let key = SecretKey::from_slice(private_key_bytes.as_slice())
                .map_err(Error::invalid_key_raw)?;
            let (_, sig_bytes) = s
                .sign_ecdsa_recoverable(&sign_msg, &key)
                .serialize_compact();
            Ok(sig_bytes.to_vec())
        }
        AddressType::Cosmos | AddressType::Ethermint { .. } => {
            let signing_key =
                SigningKey::from_bytes(private_key_bytes.as_slice()).map_err(Error::invalid_key)?;
            let signature: Signature = signing_key.sign(&msg);
            Ok(signature.as_ref().to_vec())
        }
    }
}

/// Return an address from a Public Key
fn get_address(pk: ExtendedPubKey, at: &AddressType) -> Vec<u8> {
    match at {
        AddressType::Ethermint { ref pk_type } if pk_type.ends_with(".ethsecp256k1.PubKey") => {
            let public_key = pk.public_key.serialize_uncompressed();
            // 0x04 is [SECP256K1_TAG_PUBKEY_UNCOMPRESSED](https://github.com/bitcoin-core/secp256k1/blob/d7ec49a6893751f068275cc8ddf4993ef7f31756/include/secp256k1.h#L196)
            debug_assert_eq!(public_key[0], 0x04);

            let output = keccak256_hash(&public_key[1..]);
            // right-most 20-bytes from the 32-byte keccak hash
            // (see https://kobl.one/blog/create-full-ethereum-keypair-and-address/)
            output[12..].to_vec()
        }
        AddressType::Cosmos | AddressType::Ethermint { .. } => {
            let mut hasher = Sha256::new();
            hasher.update(pk.to_pub().to_bytes().as_slice());

            // Read hash digest over the public key bytes & consume hasher
            let pk_hash = hasher.finalize();

            // Plug the hash result into the next crypto hash function.
            use ripemd160::Digest;
            let mut rip_hasher = Ripemd160::new();
            rip_hasher.update(pk_hash);
            let rip_result = rip_hasher.finalize();

            rip_result.to_vec()
        }
    }
}

fn decode_bech32(input: &str) -> Result<Vec<u8>, Error> {
    use bech32::FromBase32;

    let bytes = bech32::decode(input)
        .and_then(|(_, data, _)| Vec::from_base32(&data))
        .map_err(Error::bech32_account)?;

    Ok(bytes)
}

fn keccak256_hash(bytes: &[u8]) -> Vec<u8> {
    let mut hasher = Keccak::v256();
    hasher.update(bytes);
    let mut resp = vec![0u8; 32];
    hasher.finalize(&mut resp);
    resp
}

fn standard_path_to_derivation_path(path: &StandardHDPath) -> DerivationPath {
    use bitcoin::util::bip32::ChildNumber;

    let child_numbers = vec![
        ChildNumber::from_hardened_idx(path.purpose().as_value().as_number())
            .expect("Purpose is not Hardened"),
        ChildNumber::from_hardened_idx(path.coin_type()).expect("Coin Type is not Hardened"),
        ChildNumber::from_hardened_idx(path.account()).expect("Account is not Hardened"),
        ChildNumber::from_normal_idx(path.change()).expect("Change is Hardened"),
        ChildNumber::from_normal_idx(path.index()).expect("Index is Hardened"),
    ];

    DerivationPath::from(child_numbers)
}
