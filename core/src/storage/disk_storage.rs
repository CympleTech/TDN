use std::path::PathBuf;

use crate::actor::prelude::{Actor, Addr, Context, Handler};

use super::disk_db::DiskDatabase;
use super::entity::{Entity, EntityDelete, EntityRead, EntityWrite};

pub struct DiskStorageActor {
    db: DiskDatabase,
}

impl DiskStorageActor {
    pub fn run(path: Option<PathBuf>) -> Addr<Self> {
        Self::create(|ctx: &mut Context<Self>| {
            ctx.set_mailbox_capacity(100);
            DiskStorageActor {
                db: DiskDatabase::new(path),
            }
        })
    }
}

impl Actor for DiskStorageActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("DEBUG: Storage is started!");
    }
}

/// handle read message
impl<E: Entity> Handler<EntityRead<E>> for DiskStorageActor {
    type Result = Result<E, ()>;

    fn handle(&mut self, msg: EntityRead<E>, _ctx: &mut Self::Context) -> Self::Result {
        self.db.read_entity::<E>(msg.0)
    }
}

/// handle write message
impl<E: Entity> Handler<EntityWrite<E>> for DiskStorageActor {
    type Result = ();

    fn handle(&mut self, msg: EntityWrite<E>, _ctx: &mut Self::Context) -> Self::Result {
        self.db.write_entity(msg.0)
    }
}

/// handle delete message
impl<E: Entity> Handler<EntityDelete<E>> for DiskStorageActor {
    type Result = Result<E, ()>;

    fn handle(&mut self, msg: EntityDelete<E>, _ctx: &mut Self::Context) -> Self::Result {
        self.db.delete_entity(msg.0)
    }
}
