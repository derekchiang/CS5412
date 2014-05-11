#![crate_id = "paxos#0.1"]
#![comment = "An implementation of the multi-Paxos consensus protocol."]
#![license = "MIT/ASL2"]
#![crate_type = "lib"]

#![feature(macro_rules)]

#![allow(dead_code)]

// TODO: get rid of this once msgpack has got rid of owned vector
#![allow(deprecated_owned_vector)]

#![feature(phase)]
#![phase(syntax, link)]
extern crate collections;
extern crate log;
extern crate serialize;
extern crate uuid;
extern crate rand;
extern crate busybee;
extern crate sync;
extern crate time;

pub use common::StateMachine;
pub use common::DataConstraint;
pub use common::Command;
pub use common::CommandID;
pub use common::ServerID;
pub use client::Client;

use replica::Replica;
use acceptor::Acceptor;
use leader::Leader;
use client::Client;

pub fn new_cluster<'a, T: DataConstraint<'a>, X: StateMachine<T>>() -> Cluster<T> {
    let replica_ids = vec!(1u64 << 32, 2u64 << 32);
    let acceptor_ids = vec!(3u64 << 32, 4u64 << 32, 5u64 << 32);
    let leader_ids = vec!(6u64 << 32);

    for aid in acceptor_ids.clone().move_iter() {
        let acceptor = Acceptor::<T>::new(aid);
        spawn(proc() {
            acceptor.run();
        });
    }

    for lid in leader_ids.clone().move_iter() {
        let leader = Leader::<T>::new(lid, acceptor_ids.clone(), replica_ids.clone());
        spawn(proc() {
            leader.run();
        });
    }

    for rid in replica_ids.clone().move_iter() {
        let replica = Replica::<X, T>::new(rid, leader_ids.clone());
        spawn(proc() {
            replica.run();
        });
    }

    return Cluster {
        replicas: replica_ids
    };
}

pub struct Cluster<T> {
    replicas: Vec<ServerID>
}

impl<'a, T: DataConstraint<'a>> Cluster<T> {
    pub fn new_client(&self, sid: ServerID) -> Client<T> {
        return Client::<T>::new(sid, self.replicas.clone());
    }
}

mod common;
mod replica;
mod acceptor;
mod leader;
mod scout;
mod commander;
mod test;
mod client;

