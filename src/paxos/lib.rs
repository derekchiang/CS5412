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

pub use common::StateMachine;
pub use common::DataConstraint;
pub use client::Client;

// pub fn create_cluster<'a, T: DataConstraint<'a>, X: StateMachine>() {
//     let replica_ids = vec!(2, 3);
//     let acceptor_ids = vec!(4, 5, 6);
//     let leader_ids = vec!(7);

//     for rid in replica_ids.iter() {
//         let replica = Replica::<X, T>::new(rid, leader_ids.clone());
//         spawn(proc() {
//             replica.run();
//         });
//     }

//     for aid in acceptor_ids.iter() {
//         let acceptor = Acceptor::new(aid);
//         spawn(proc() {
//             acceptor.run();
//         });
//     }

//     for lid in leader_ids.iter() {
//         let leader = Leader::new(lid);
//         spawn(proc() {
//             leader.run();
//         });
//     }

//     return Client {
//         replicas: replica_ids
//     }
// }

mod common;
mod replica;
mod acceptor;
mod leader;
mod scout;
mod commander;
mod test;
mod client;

