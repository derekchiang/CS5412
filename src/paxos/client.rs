#[phase(syntax, link)] extern crate log;

use time::precise_time_ns;

use collections::hashmap::HashMap;

use common;
use common::{DataConstraint, Message, Request, Response, ServerID, Command};

use busybee::{Busybee, BusybeeMapper};

pub struct Client<T> {
    id: ServerID,
    bb: Busybee,
    replicas: Vec<ServerID>,
    calls_chans_tx: SyncSender<(u64, Sender<T>)>,
}

impl<'a, T: DataConstraint<'a>> Client<T> {
    pub fn new(sid: ServerID, rids: Vec<ServerID>) -> Client<T> {
        let bb = Busybee::new(sid, common::lookup(sid), 0, BusybeeMapper::new(common::lookup));
        let (calls_chans_tx, calls_chans_rx) = sync_channel(0);

        let client = Client {
            id: sid,
            bb: bb,
            replicas: rids,
            calls_chans_tx: calls_chans_tx,
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
                                match chans_map.pop(&comm_id) {
                                    Some(resp_tx) => resp_tx.send(resp),
                                    None => error!("Received a command that is not being waited for: {}", comm_id)
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
            id: precise_time_ns(),  // weird bug happens if we simply generate a random u64... seems like a Rust bug.
            name: name,
            args: args
        };

        // We want to set up receivers before sending the command
        let (tx, rx) = channel();
        self.calls_chans_tx.send((comm.id, tx));

        for replica in self.replicas.iter() {
            self.bb.send_object::<Message<T>>(replica.clone(), Request(comm.clone()));
        }

        return rx;
    }
}