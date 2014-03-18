extern crate msgpack;
extern crate serialize;
extern crate russenger;
extern crate rand;

use std::fmt::Show;
use std::io::net::ip::SocketAddr;
use rand::random;

use serialize::{Encodable, Decodable};

use msgpack::{Encoder, Decoder};

use common::{LeaderId, Proposal, BallotNum, SlotNum, Message, Pvalue, Decision};
use common::{P1a, P1b, P2a, P2b};

pub struct Leader<X> {
    id: LeaderId,
    ballot_num: BallotNum,
    active: bool,
    lu_slot_num: SlotNum, // lowest undecided slot number
    proposals: ~[Proposal],
    acceptors: ~[SocketAddr],
    // Communication endpoints with the outside world (acceptors, replicas)
    tx: Sender<(SocketAddr, Message<X>)>,
    rx: Receiver<(SocketAddr, Message<X>)>,
    // Communication endpoints with the spawned commanders and scouts
    inner_tx: Sender<Message<X>>,
    inner_rx: Receiver<Message<X>>,
}

impl<'a, X: Send + Show + Encodable<Encoder<'a>> + Decodable<Decoder<'a>>> Leader<X> {
    pub fn new(addr: SocketAddr, acceptors: ~[SocketAddr]) -> Leader<X> {
        let (tx, rx) = russenger::new::<Message<X>>(addr.clone());
        let (inner_tx, inner_rx) = channel();
        let rand_id = random();  // TODO: leader IDs could collide.  This would cause problem.
        Leader {
            id: rand_id,
            ballot_num: (0u, rand_id),
            active: false,
            lu_slot_num: 0,
            proposals: ~[],
            acceptors: acceptors,
            tx: tx,
            rx: rx,
            inner_tx: inner_tx,
            inner_rx: inner_rx,
        }
    }

    pub fn run(mut ~self) {
        self.spawn_scout(self, self.acceptors, ballot_num);
        loop {
            let msg = self.inner_rx.recv();
            match msg {
                Proposal(s_num, comm) => {
                    if (!(chk_contains_slot(self.proposals, s_num))) { //if proposals does not contain this slot number already
                        self.proposals = self.proposals.append(Proposal(s_num, comm));
                        if (self.active) {
                            spawn_commander(self, self.acceptors, self.replicas, (self.ballot_num, s_num, comm));
                        }
                    }
                }
                Adopted(b_num, pvalues) => {
                    pvalues = pmax(pvalues);
                    self.proposals = p_update(self.proposals, pvalues);
                    for (s, p) in self.proposals {
                        spawn_commander(self, self.acceptors, self.replicas, (self.ballot_num, s, p));
                    }
                    active = true;
                }
                Preempted((b_num, l_id)) => {
                    let (curr_num, my_id) = self.ballot_num;
                    if (b_num > curr_num) {
                        active = false;
                        self.ballot_num = (b_num + 1, self.id);
                        spawn_scout(self, self.acceptors, self.ballot_num);
                    }
                }
            }
            
        }
    }

    //takes a list of proposals and a slot number, returns true if that list contains a proposal with that slot number
    fn chk_contains_slot(proposals: ~[Proposal], s: uint) { //check these type annotations
        for (s_num, comm) in proposals.iter() {
            if (s_num == s) {
                return true;
            }
        }
    }

    fn spawn_commander(&mut self, acceptors: ~[str], replicas: ~[str], pval: Pvalue) { //check these type annotations

        let waiting_for = acceptors.clone();
        for acc in acceptors.iter() {
            self.tx.send((acc.clone(), P2a(pval)));
        }
        loop {
            let (acc, msg) = self.inner_rx.recv(); //somehow?
            match msg {
                P2b((b_num, l_id)) => {
                    let ((bal, lid), slt, comm) = pval; //make sure this doesnt cause problems with the ownerships
                    if (b_num == bal) {
                        waiting_for.remove(acc); //fix
                        if (len(waiting_for) < len(acceptors) / 2) { //fix
                            for rep in replicas.iter() {
                                self.tx.send((rep.clone(), Decision(slt, comm)));
                            }
                            exit() //what do i do here
                        }
                    } else {
                        self.inner_tx.send(Preempted((b_num, l_id))); //tell leader that this one has been preempted
                        exit();
                    }
                }
            } 
        }

    }

    fn spawn_scout(&mut self, acceptors: ~[str], bal: BallotNum) {
        let waiting_for = acceptors.clone();
        let pvalues = ~[];
        for acc in acceptors.iter() {
            self.tx.send((acc.clone(), P1a((bal, self.id), self.lu_slot_num))); //hopefully correct parameters for the P1a
        }
        loop {
            let (acc, msg) = self.inner_rx.recv(); //probably wrong
            match msg {
                P1b((b_num, l_id), pvals) => {
                    if (b_num == bal) {
                        pvalues = pvalues.union(pvals);
                        waiting_for.remove(acc);
                        if (len(waiting_for) < len(acceptors) / 2) { //fix
                            self.inner_tx.send(Adopted(b_num, pvalues));
                            exit();
                        }
                    } else {
                        self.inner_tx.send(Preempted(b_num, l_id)); //tell leader that this one has been preempted
                        exit();
                    }

                }
            }


        }


    }
}