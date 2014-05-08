#[phase(syntax, link)] extern crate log;

use collections::hashmap::HashMap;

use common;
use common::{DataConstraint, Message, Request, Response, ServerID, Command};

use busybee::{Busybee, BusybeeMapper};

pub struct Client<T> {
    id: ServerID,
    bb: Busybee,
    next_comm_id: u64,
    replicas: Vec<ServerID>,
    calls_chans_tx: Sender<(u64, Sender<T>)>
}

impl<'a, T: DataConstraint<'a>> Client<T> {
    pub fn new(sid: ServerID, rids: Vec<ServerID>) -> Client<T> {
        let bb = Busybee::new(sid, common::lookup(sid), 0, BusybeeMapper::new(common::lookup));
        let (calls_chans_tx, calls_chans_rx) = channel();

        let client = Client {
            id: sid,
            bb: bb,
            next_comm_id: 0,
            replicas: rids,
            calls_chans_tx: calls_chans_tx
        };

        let bb2 = bb.clone();
        spawn(proc() {
            let (bb_tx, bb_rx) = channel();
            spawn(proc() {
                let mut bb = bb2;
                loop {
                    match bb.recv_object::<Message<T>>() {
                        Ok((sid, msg)) => bb_tx.send((sid, msg)),
                        Err(e) => error!("ERROR: {}", e)
                    }
                }
            });

            let mut chans_map = HashMap::new();
            loop {
                select!(
                    (comm_id, resp_tx) = calls_chans_rx.recv() => {
                        chans_map.insert(comm_id, resp_tx);
                    },

                    (from, msg) = bb_rx.recv() => {
                        info!("client {}: recv {} from {}", sid, msg, from);
                        // TODO: verify that the message comes from a leader
                        match msg {
                            Response(comm_id, resp) => {
                                println!("BP0");
                                match chans_map.pop(&comm_id) {
                                    Some(resp_tx) => resp_tx.send(resp),
                                    None => {}
                                };
                            }
                            
                            x => info!("wrong message: {}", x)
                        }
                    }
                );
            }
        });

        return client;
    }

    pub fn call(&mut self, name: ~str, args: Vec<~str>) -> Receiver<T> {
        let comm = Command {
            from: self.id,
            id: self.next_comm_id,
            name: name,
            args: args
        };
        self.next_comm_id += 1;

        // We want to set up receivers before sending the command
        let (tx, rx) = channel();
        self.calls_chans_tx.send((comm.id, tx));

        for replica in self.replicas.iter() {
            self.bb.send_object::<Message<T>>(replica.clone(), Request(comm.clone()));
        }

        return rx;
    }
}