use std::sync::Arc;
use derive_more::{Deref, DerefMut};
use pickledb::PickleDb;
use serenity::prelude::TypeMapKey;
use tokio::sync::Mutex;

#[derive(Clone, Deref, DerefMut)]
pub struct DataStore(pub Arc<Mutex<PickleDb>>);

impl TypeMapKey for DataStore {
    type Value = DataStore;
}
