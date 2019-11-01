use rcmixed::store::{decrypt_secret_key, encrypt_secret_key};
use rcmixed::traits::PublicKeyAlgorithm;
use serde_derive::{Deserialize, Serialize};
use std::marker::PhantomData;

use teatree::actor::prelude::Addr;
use teatree::primitives::functions::get_default_storage_path;
use teatree::primitives::types::GroupID;
use teatree::storage::{DiskDatabase, DiskStorageActor, Entity, EntityWrite};
use teatree::traits::propose::Peer;

pub struct KeyStore<P: 'static + Peer> {
    pk: P::PublicKey,
    psk: P::PrivateKey,
    storage: Addr<DiskStorageActor>,
    password: String,
}

impl<P: 'static + Peer> KeyStore<P> {
    pub fn load(id: &GroupID, password: String) -> Self {
        let mut path = get_default_storage_path();
        path.push("keys");
        path.push(format!("{}", id));
        let db = DiskDatabase::new(Some(path.clone()));

        let (pk, psk) = if let Ok(main) = db.read_entity::<MainKeyStore<P>>(id.to_string()) {
            if let Ok(key) = db.read_entity::<KeypairStore<P>>(main.1.to_string()) {
                let secret_key = decrypt_secret_key::<PrivateKeyStore<P>>(key.1, password.clone());
                if secret_key.is_some() {
                    (key.0, secret_key.unwrap())
                } else {
                    panic!("CANNOT LOAD PUBLICK & PRIVATEKEY");
                }
            } else {
                panic!("CANNOT LOAD PUBLICK & PRIVATEKEY");
            }
        } else {
            let (pk, psk) = P::generate();
            db.write_entity(MainKeyStore::<P>(id.clone(), pk.clone()));

            let secret_key: <PrivateKeyStore<P> as PublicKeyAlgorithm>::SecretKey = psk.clone();
            let ciphertext =
                encrypt_secret_key::<PrivateKeyStore<P>>(&secret_key, password.clone());

            db.write_entity(KeypairStore::<P>(pk.clone(), ciphertext));
            (pk, psk)
        };

        drop(db);

        let storage = DiskStorageActor::run(Some(path));

        Self {
            pk,
            psk,
            storage,
            password,
        }
    }

    pub fn key(&self) -> (&P::PublicKey, &P::PrivateKey) {
        (&self.pk, &self.psk)
    }

    pub fn async_store(&self, pk: P::PublicKey, psk: P::PrivateKey) {
        let secret_key: <PrivateKeyStore<P> as PublicKeyAlgorithm>::SecretKey = psk;
        let ciphertext =
            encrypt_secret_key::<PrivateKeyStore<P>>(&secret_key, self.password.clone());

        self.storage
            .do_send(EntityWrite(KeypairStore::<P>(pk, ciphertext)));
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct MainKeyStore<P: 'static + Peer>(GroupID, P::PublicKey);

impl<P: 'static + Peer> Entity for MainKeyStore<P> {
    type Key = String;

    fn key(&self) -> Self::Key {
        self.0.to_string()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct KeypairStore<P: 'static + Peer>(P::PublicKey, Vec<u8>);

impl<P: 'static + Peer> Entity for KeypairStore<P> {
    type Key = String;

    fn key(&self) -> Self::Key {
        self.0.to_string()
    }
}

pub struct PrivateKeyStore<P: Peer>(PhantomData<P>);

impl<P: Peer> PublicKeyAlgorithm for PrivateKeyStore<P> {
    type PublicKey = P::PublicKey;
    type SecretKey = P::PrivateKey;
    const SECRET_KEY_LENGTH: usize = P::PRIVATE_KEY_LENGTH;

    fn encrypt(_plain: &[u8], _public_key: &Self::PublicKey) -> Vec<u8> {
        vec![]
    }

    fn decrypt(_cipher: &[u8], _secret_key: &Self::SecretKey) -> Vec<u8> {
        vec![]
    }

    fn from_bytes(bytes: &[u8]) -> Option<Self::SecretKey> {
        P::private_key_from_bytes(bytes)
    }

    fn to_bytes(sign_key: &Self::SecretKey) -> Vec<u8> {
        P::private_key_to_bytes(sign_key)
    }
}
