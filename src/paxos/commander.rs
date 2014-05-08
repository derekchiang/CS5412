#[phase(syntax, link)] extern crate log;

use std::iter::FromIterator;

use collections::hashmap::HashSet;

use busybee::Busybee;

use common::{DataConstraint, Message, ServerID, Pvalue, Decision, Preempted};
use common::{P2a, P2b};

pub struct Commander<X> {
    id: ServerID,
    leader_id: ServerID,
	acceptors: Vec<ServerID>,
	replicas: Vec<ServerID>,
	pval: Pvalue,
    bb: Busybee,
    rx: Receiver<(ServerID, Message<X>)>,
}

impl<'a, X: DataConstraint<'a>> Commander<X> {
	pub fn new(commander_id: ServerID, leader_id: ServerID, acceptors: Vec<ServerID>,
        replicas: Vec<ServerID>, pval: Pvalue, bb: Busybee, rx: Receiver<(ServerID, Message<X>)>) -> Commander<X> {
        Commander {
        	id: commander_id,
            leader_id: leader_id,
            acceptors: acceptors,
            replicas: replicas,
            pval: pval,
            bb: bb,
            rx: rx,
        }
	}

	pub fn run(mut self) {
        let mut waitfor: HashSet<ServerID> = FromIterator::from_iter(self.acceptors.clone().move_iter());
        for acceptor in self.acceptors.iter() {
            self.bb.send_object::<Message<X>>(acceptor.clone(), P2a(self.id, self.pval.clone()));
        }

        loop {
            let (acceptor_id, msg) = self.rx.recv();
            info!("commander {}: recv {} from {}", self.id, msg, acceptor_id);
            match msg {
                P2b(_, ballot_num) => {
                    let (bnum, slot_num, ref comm) = self.pval;
                    if bnum == ballot_num {
                        waitfor.remove(&acceptor_id);
                        if waitfor.len() < (self.acceptors.len() + 1) / 2 {
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