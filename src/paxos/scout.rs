use std::fmt::Show;
use std::io::IoError;
use std::iter::FromIterator;

use collections::hashmap::HashSet;

use serialize::json;
use serialize::{Encodable, Decodable};
use serialize::json::{Encoder, Decoder};

use busybee::{Busybee, BusybeeMapper};

use common;
use common::{ServerID, BallotNum, Message, Pvalue, Adopted, Preempted};
use common::{P1a, P1b};

pub struct Scout {
    id: ServerID,
    leader_id: ServerID,
    ballot_num: BallotNum,
    acceptors: ~[ServerID],
    bb: Busybee,
}

impl<'a, X: Send + Show + Encodable<Encoder<'a>, IoError> + Decodable<Decoder, json::Error>> Scout {

    pub fn new(scout_id: ServerID, leader_id: ServerID, acceptors: ~[ServerID], bnum: BallotNum) -> Scout {
        let bb = Busybee::new(scout_id, common::lookup(scout_id), 1, BusybeeMapper::new(common::lookup));
        Scout {
            id: scout_id,
            leader_id: leader_id,
            ballot_num: bnum,
            acceptors: acceptors,
            bb: bb,
        }
    }

    pub fn run(mut ~self) {
        for acc in self.acceptors.iter() {
            self.bb.send_object::<Message<X>>(acc.clone(), P1a(self.ballot_num));
        }
        let mut waitfor: HashSet<ServerID> = FromIterator::from_iter(self.acceptors.clone().move_iter());
        let mut pvalues: HashSet<Pvalue> = HashSet::new();
        loop {
            let (acceptor_id, msg): (ServerID, Message<X>) = self.bb.recv_object().unwrap();
            match msg {
                P1b(bnum, accepted_pvals) => {
                    if bnum == self.ballot_num {
                        for pval in accepted_pvals.move_iter() {
                            pvalues.insert(pval);
                        }
                        waitfor.remove(&acceptor_id);
                        if waitfor.len() < (self.acceptors.len() / 2) {
                            let pvalues_vec = FromIterator::from_iter(pvalues.move_iter());
                            self.bb.send_object::<Message<X>>(self.leader_id, Adopted(bnum, pvalues_vec));
                            return;
                        }
                    } else {
                        self.bb.send_object::<Message<X>>(self.leader_id, Preempted(bnum));
                        return;
                    }

                }

                _ => {} //need some debug statement here
            }


        }


    }
}