extern crate msgpack;
extern crate serialize;
extern crate russenger;
extern crate rand;

use std::fmt::Show;
use std::io::net::ip::SocketAddr;
use rand::random;

use serialize::{Encodable, Decodable};

use msgpack::{Encoder, Decoder};

use common::{LeaderId, Proposal, BallotNum, SlotNum, Message, Pvalue, Decision, Propose, Adopted, Preempted};
use common::{P1a, P1b, P2a, P2b};

pub struct Leader<X> {
    id: LeaderId,
    ballot_num: BallotNum,
    active: bool,
    lu_slot_num: SlotNum, // lowest undecided slot number
    proposals: ~[Proposal],
    acceptors: ~[SocketAddr],
    replicas: ~[SocketAddr],
    // Communication endpoints with the outside world (acceptors, replicas)
    tx: Sender<(SocketAddr, Message<X>)>,
    rx: Receiver<(SocketAddr, Message<X>)>,
    // Communication endpoints with the spawned commanders and scouts
    inner_tx: Sender<Message<X>>,
    inner_rx: Receiver<Message<X>>,
}

impl<'a, X: Send + Show + Encodable<Encoder<'a>> + Decodable<Decoder<'a>>> Leader<X> {
    pub fn new(addr: SocketAddr, acceptors: ~[SocketAddr], replicas : ~[SocketAddr]) -> Leader<X> {
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
            replicas: replicas,
            tx: tx,
            rx: rx,
            inner_tx: inner_tx,
            inner_rx: inner_rx,
        }
    }

    pub fn run(mut ~self) {
        self.spawn_scout(self.acceptors, self.ballot_num);
        loop {
            let msg = self.inner_rx.recv();
            match msg {
                Propose((s_num, comm)) => {
                    if (!(self.chk_contains_slot(self.proposals, s_num))) { //if proposals does not contain this slot number already
                        self.proposals.push((s_num, comm));
                        if (self.active) {
                            self.spawn_commander(self.acceptors, self.replicas, (self.ballot_num, s_num, comm));
                        }
                    }
                }
                Adopted(b_num, pvalues) => { //maybe check if this ballot number is the right one
                    let max_pvalues = self.pmax(pvalues);
                    self.proposals = self.p_update(self.proposals, max_pvalues); // need to find out how to resolve this
                    for (s, p) in self.proposals.iter() {
                        self.spawn_commander(self.acceptors, self.replicas, (self.ballot_num, s, p));
                    }
                    self.active = true;
                }
                Preempted((b_num, l_id)) => {
                    let (curr_num, my_id) = self.ballot_num;
                    if (b_num > curr_num) {
                        self.active = false;
                        self.ballot_num = (b_num + 1, self.id);
                        self.spawn_scout(self.acceptors, self.ballot_num);
                    }
                }
            }
            
        }
    }

    //takes a list of proposals and a slot number, returns true if that list contains a proposal with that slot number
    fn chk_contains_slot(~self, proposals: ~[Proposal], s: uint) -> bool{ //check these type annotations
        for pr in proposals.iter() {
            let (s_num, comm) = pr;
            if (s_num == s) {
                return true;
            }
        }
        return false;
    }

    fn pmax(&self, pvals: ~[Pvalue]) -> ~[Proposal] {
        return ~[];
    }

    fn p_update(&self, orig: ~[Proposal], updates: ~[Proposal]) -> ~[Proposal] {
       return orig;
    }

    fn union<T>(&self, a: ~[T], b: ~[T]) -> ~[T] {
        return a;
    }

    fn remove_ele<T>(&self, lst: ~[T], ele: T) -> ~[T] {
        return lst;
    }

    fn spawn_commander(&mut self, acceptors: ~[SocketAddr], replicas: ~[SocketAddr], pval: Pvalue) { //check these type annotations

        let waiting_for = acceptors.clone();
        for acc in acceptors.iter() {
            self.tx.send((acc.clone(), P2a(pval)));
        }
        loop {
            let (acc, msg) = self.rx.recv(); //somehow?
            match msg {
                P2b((b_num, l_id)) => {
                    let ((bal, lid), slt, comm) = pval; //make sure this doesnt cause problems with the ownerships
                    if (b_num == bal) {
                        self.remove_ele(waiting_for, acc);
                        if (waiting_for.iter().len() < acceptors.iter().len() / 2) { //fix
                            for rep in replicas.iter() {
                                self.tx.send((rep.clone(), Decision((slt, comm))));
                            }
                            break;
                        }
                    } else {
                        self.inner_tx.send(Preempted((b_num, l_id))); //tell leader that this one has been preempted
                        break;
                    }
                }
            } 
        }

    }

    fn spawn_scout(&mut self, acceptors: ~[SocketAddr], bal: BallotNum) {
        let waiting_for = acceptors.clone();
        let pvalues = ~[];
        for acc in acceptors.iter() {
            self.tx.send((acc.clone(), P1a(bal, self.lu_slot_num))); //hopefully correct parameters for the P1a
        }
        loop {
            let (acc, msg) = self.rx.recv(); //probably wrong
            match msg {
                P1b((b_num, l_id), pvals) => {
                    let (my_b, my_id) = bal;
                    if (b_num == my_b) {
                        pvalues = self.union(pvalues, pvals);
                        self.remove_ele(waiting_for, acc);
                        if (waiting_for.iter().len() < acceptors.iter().len() / 2) { //fix
                            self.inner_tx.send(Adopted((b_num, l_id), pvalues));
                            break;
                        }
                    } else {
                        self.inner_tx.send(Preempted((b_num, l_id))); //tell leader that this one has been preempted
                        break;
                    }

                }
            }


        }


    }
}