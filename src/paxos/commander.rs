use std::fmt::Show;
use std::io::IoError;
use std::iter::FromIterator;

use collections::hashmap::HashSet;

use serialize::json;
use serialize::{Encodable, Decodable};
use serialize::json::{Encoder, Decoder};

use busybee::{Busybee, BusybeeMapper};

use common;
use common::{Message, ServerID, Pvalue, Decision, Preempted};
use common::{P2a, P2b};

pub struct Commander {
    id: ServerID,
    leader_id: ServerID,
	acceptors: ~[ServerID],
	replicas: ~[ServerID],
	pval: Pvalue,
    bb: Busybee,
}

impl<'a, X: Send + Show + Encodable<Encoder<'a>, IoError> + Decodable<Decoder, json::Error>> Commander {
	pub fn new(commander_id: ServerID, leader_id: ServerID, acceptors: ~[ServerID],
        replicas: ~[ServerID], pval: Pvalue) -> Commander {
		let bb = Busybee::new(commander_id, common::lookup(commander_id), 1, BusybeeMapper::new(common::lookup));
        Commander {
        	id: commander_id,
            leader_id: leader_id,
            acceptors: acceptors,
            replicas: replicas,
            pval: pval,
            bb: bb,
        }
	}

	pub fn run(mut ~self) {
        let mut waitfor: HashSet<ServerID> = FromIterator::from_iter(self.acceptors.clone().move_iter());
        for acceptor in self.acceptors.iter() {
            self.bb.send_object::<Message<X>>(acceptor.clone(), P2a(self.pval.clone()));
        }

        loop {
            let (acceptor, msg): (ServerID, Message<X>) = self.bb.recv_object().unwrap();
            match msg {
                P2b(ballot_num) => {
                    let (bnum, slot_num, ref comm) = self.pval;
                    if bnum == ballot_num {
                        waitfor.remove(&acceptor);
                        if waitfor.len() < self.acceptors.len() / 2 {
                            for replica in self.replicas.iter() {
                                self.bb.send_object::<Message<X>>(replica.clone(), Decision((slot_num, comm.clone())));
                            }
                            return;
                        }
                    } else {
                        self.bb.send_object::<Message<X>>(self.leader_id, Preempted(ballot_num));
                        return;
                    }
                }

                _ => {} //need some debug statement here
            }
        }
	}
}