#![allow(non_camel_case_types)]

extern crate paxos;
extern crate collections;
extern crate serialize;

use std::io::timer::sleep;

use collections::hashmap::HashMap;

use paxos::StateMachine;
use paxos::Command;
use paxos::ServerID;

struct Lock<T> {
    holder: Option<ServerID>,
    waiting: Vec<ServerID>,
    waiting_txs: HashMap<ServerID, Sender<T>>
}

#[deriving(Encodable, Decodable, Show)]
enum Response {
    LOCK_SUCCESS,
    UNLOCK_SUCCESS,
    NOT_LOCKED_BY_YOU,
    UNEXPECTED_COMMAND
}

impl StateMachine<Response> for Lock<Response> {
    fn new() -> Lock<Response> {
        return Lock {
            holder: None,
            waiting: vec!(),
            waiting_txs: HashMap::new()
        }
    }

    fn destroy(self) {}

    fn clone(&self) -> Lock<Response> {
        Lock {
            holder: self.holder.clone(),
            waiting: self.waiting.clone(),
            waiting_txs: self.waiting_txs.clone()
        }
    }

    fn invoke_command(&mut self, res_tx: Sender<Response>, comm: Command) {
        match comm.name.as_slice() {
            "lock" => {
                if self.holder.is_none() {
                    self.holder = Some(comm.from);
                    res_tx.send(LOCK_SUCCESS);
                } else {
                    self.waiting.push(comm.from);
                    self.waiting_txs.insert(comm.from, res_tx);
                }
            }

            "unlock" => {
                if self.holder.is_none() || self.holder.unwrap() != comm.from {
                    res_tx.send(NOT_LOCKED_BY_YOU);
                } else {
                    match self.waiting.shift() {
                        Some(s) => {
                            self.holder = Some(s);
                            let res_tx = self.waiting_txs.pop(&s).unwrap();
                            res_tx.send(LOCK_SUCCESS);
                        }

                        None => {
                            self.holder = None;
                        }
                    }
                    res_tx.send(UNLOCK_SUCCESS);
                }
            }

            _ => {
                res_tx.send(UNEXPECTED_COMMAND);
            }
        }
    }
}

fn main() {
    let cluster = paxos::new_cluster::<Response, Lock<Response>>();

    let client1 = cluster.new_client(7u64 << 32);
    let client2 = cluster.new_client(8u64 << 32);

    spawn(proc() {
        let mut client1 = client1;
        let res_rx = client1.call(~"lock", vec!());
        println!("Client 1: {}", res_rx.recv());
        sleep(2000);
        let res_rx = client1.call(~"unlock", vec!());
        println!("Client 1: {}", res_rx.recv());
    });

    spawn(proc() {
        let mut client2 = client2;
        let res_rx = client2.call(~"lock", vec!());
        println!("Client 2: {}", res_rx.recv());
        sleep(2000);
        let res_rx = client2.call(~"unlock", vec!());
        println!("Client 2: {}", res_rx.recv());
    });
}