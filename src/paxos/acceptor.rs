#[phase(syntax, link)] extern crate log;

use common;
use common::{DataConstraint, ServerID, BallotNum, Pvalue, Message};
use common::{P1a, P1b, P2a, P2b};

use busybee::{Busybee, BusybeeMapper};

pub struct Acceptor<X> {
    id: ServerID,
    ballot_num: BallotNum,
    accepted: Vec<Pvalue>,
    bb: Busybee,
}

impl<'a, X: DataConstraint<'a>> Acceptor<X> {
    pub fn new(sid: ServerID) -> Acceptor<X> {
        let bb = Busybee::new(sid, common::lookup(sid), 0, BusybeeMapper::new(common::lookup));
        Acceptor {
            id: sid,
            ballot_num: (0u64, 0u64),
            accepted: vec!(),
            bb: bb,
        }
    }

    pub fn run(mut self) {
        loop {
            match self.bb.recv_object::<Message<X>>() {
                Ok((from, msg)) => {
                    info!("acceptor {}: recv {} from {}", self.id, msg, from);
                    match msg {
                        P1a(sid, bnum) => {
                            if bnum > self.ballot_num {
                                self.ballot_num = bnum;
                            }

                            self.bb.send_object::<Message<X>>(from, P1b(sid, self.ballot_num, self.accepted.clone()));
                        }

                        P2a(sid, pvalue) => {
                            let (b, _, _) = pvalue.clone();
                            if b >= self.ballot_num {
                                self.ballot_num = b;
                                self.accepted.push(pvalue);
                            }
                            self.bb.send_object::<Message<X>>(from, P2b(sid, self.ballot_num));
                        }

                        _ => error!("ERROR: wrong message {} from {}", msg, from)
                    }
                }

                Err(e) => error!("Error: {}", e)
            }
        }
    }
}