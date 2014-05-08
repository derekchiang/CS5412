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
                let (sid, msg): (ServerID, Message<T>) = bb.recv_object().unwrap();
                bb_tx.send((sid, msg));
            });

            let mut chans_map = HashMap::new();
            loop {
                select!(
                    (comm_id, resp_tx) = calls_chans_rx.recv() => {
                        chans_map.insert(comm_id, resp_tx);
                    },

                    (_, msg) = bb_rx.recv() => {
                        // TODO: verify that the message comes from a leader
                        match msg {
                            Response(comm_id, resp) => {
                                let resp_tx = chans_map.get_copy(&comm_id);
                                resp_tx.send(resp);
                            }
                            
                            x => info!("wrong message: {}", x)
                        }
                    }
                )
            }
        });

        client
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