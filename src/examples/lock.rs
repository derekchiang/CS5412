#![allow(non_camel_case_types)]

extern crate paxos;
extern crate collections;
extern crate serialize;
extern crate rand;

use std::io::timer::sleep;

use collections::hashmap::HashMap;

use paxos::StateMachine;
use paxos::Command;
use paxos::ServerID;

use rand::random;

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
                    res_tx.send(UNLOCK_SUCCESS);
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
                }
            }

            _ => {
                res_tx.send(UNEXPECTED_COMMAND);
            }
        }
    }
}

// Sleep for any time between 0 and 1 second
fn random_sleep() {
    sleep(random::<u64>() % 1000);
}

fn main() {
    let cluster = paxos::new_cluster::<Response, Lock<Response>>();

    let client1 = cluster.new_client(7u64 << 32);
    let client2 = cluster.new_client(8u64 << 32);
    let client3 = cluster.new_client(9u64 << 32);

    spawn(proc() {
        let mut client1 = client1;
        for _ in range(0, 10) {
            let res_rx = client1.call(~"lock", vec!());
            println!("Client 1: {}", res_rx.recv());
            random_sleep();
            let res_rx = client1.call(~"unlock", vec!());
            println!("Client 1: {}", res_rx.recv());
        }
    });

    spawn(proc() {
        let mut client2 = client2;
        for _ in range(0, 10) {
            let res_rx = client2.call(~"lock", vec!());
            println!("Client 2: {}", res_rx.recv());
            random_sleep();
            let res_rx = client2.call(~"unlock", vec!());
            println!("Client 2: {}", res_rx.recv());
        }
    });

    spawn(proc() {
        let mut client3 = client3;
        for _ in range(0, 10) {
            let res_rx = client3.call(~"lock", vec!());
            println!("Client 3: {}", res_rx.recv());
            random_sleep();
            let res_rx = client3.call(~"unlock", vec!());
            println!("Client 3: {}", res_rx.recv());
        }
    });
}