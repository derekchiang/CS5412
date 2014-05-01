use std::fmt::Show;
use std::io::IoError;
use std::iter::FromIterator;

use collections::hashmap::HashSet;

use serialize::json;
use serialize::{Encodable, Decodable};
use serialize::json::{Encoder, Decoder};

use busybee::Busybee;

use common::{ServerID, BallotNum, Message, Pvalue, Adopted, Preempted};
use common::{P1a, P1b};

pub struct Scout<X> {
    id: ServerID,
    leader_id: ServerID,
    ballot_num: BallotNum,
    acceptors: Vec<ServerID>,
    bb: Busybee,
    rx: Receiver<(ServerID, Message<X>)>,
}

impl<'a, X: Send + Show + Encodable<Encoder<'a>, IoError> + Decodable<Decoder, json::Error>> Scout<X> {
    pub fn new(scout_id: ServerID, leader_id: ServerID, acceptors: Vec<ServerID>, bnum: BallotNum, bb: Busybee, rx: Receiver<(ServerID, Message<X>)>) -> Scout<X> {
        Scout {
            id: scout_id,
            leader_id: leader_id,
            ballot_num: bnum,
            acceptors: acceptors,
            bb: bb,
            rx: rx,
        }
    }

    pub fn run(mut self) {
        for acc in self.acceptors.iter() {
            self.bb.send_object::<Message<X>>(acc.clone(), P1a(self.id, self.ballot_num));
        }
        let mut waitfor: HashSet<ServerID> = FromIterator::from_iter(self.acceptors.clone().move_iter());
        let mut pvalues: HashSet<Pvalue> = HashSet::new();
        loop {
            let (acceptor_id, msg) = self.rx.recv();
            match msg {
                P1b(_, bnum, accepted_pvals) => {
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