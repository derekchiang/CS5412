extern crate msgpack;
extern crate serialize;
extern crate russenger;
extern crate rand;
extern crate collections;

use std::fmt::Show;
use std::io::net::ip::SocketAddr;
use rand::random;

use collections::hashmap::HashMap;

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
        self.spawn_scout(self.ballot_num);
        loop {
            let msg = self.inner_rx.recv();
            match msg {
                Propose((s_num, comm)) => {
                    if !(self.chk_contains_slot(&self.proposals, s_num)) { //if proposals does not contain this slot number already
                        self.proposals.push((s_num, comm.clone()));
                        if self.active {
                            spawn(self.spawn_commander((self.ballot_num, s_num, comm)));
                           /* spawn(proc() {
                                let commander = Commander::new();
                                commander.run();
                            }); */
                        }
                    }
                }
                Adopted(b_num, pvalues) => { //maybe check if this ballot number is the right one
                    let max_pvalues = self.pmax(pvalues);
                    self.proposals = self.p_update(&self.proposals, max_pvalues); // need to find out how to resolve this
                    let prop_clone = self.proposals.clone();
                    for (s, p) in prop_clone.move_iter() {
                        self.spawn_commander((self.ballot_num, s, p));
                    }
                    self.active = true;
                }
                Preempted((b_num, _)) => {
                    let (curr_num, _) = self.ballot_num;
                    if b_num > curr_num {
                        self.active = false;
                        self.ballot_num = (b_num + 1, self.id);
                        self.spawn_scout(self.ballot_num);
                    }
                }
                _ => {} //need some debug statement here 
            }
            
        }
    }

    //takes a list of proposals and a slot number, returns true if that list contains a proposal with that slot number
    fn chk_contains_slot(&self, proposals: &~[Proposal], s: SlotNum) -> bool{ //check these type annotations
        let prop_clone = proposals.clone();
        for pr in prop_clone.move_iter() {
            let (s_num, _) = pr;
            if s_num == s {
                return true;
            }
        }
        return false;
    }

    fn pmax(&self, pvals: ~[Pvalue]) -> ~[Proposal] {
        let mut new_pv : ~[Pvalue] = vec::new();
        let mut ret : ~[Proposal] = vec::new();
        for pv in pvals.move_iter() {
            new_pv = pmax_helper(new_pv, pv);
        }
        for (_, sl, comm) in new_pv.move_iter() {
            ret.push((sl, comm));
        }
        return ret;
    }

    fn pmax_helper(&self, pvals: ~[Pvalue], new_pval: Pvalue) -> ~[Pvalue] {
        let ret : ~[Pvalue] = vec::new();
        let ((bal, l), sl, comm) = new_pval;
        for ((b2, l2), sl2, c2) in pvals.iter() {
            if (sl == sl2 && bal > b2) {
                ret.push(new_pval);
            } else {
                ret.push(((b2, l2), sl2, c2));
            }
        }
        return ret;
    }

    fn p_update(&self, orig: &~[Proposal], updates: ~[Proposal]) -> ~[Proposal] {
        for pr in orig.iter() {
            if !(chk_contains_slot(updates, pr)) {
                updates.push(pr);
            }
       }
       return updates;
    }

    fn spawn_commander(&mut self, pval: Pvalue) { //change this to an actual spawn
       spawn(proc() {
        let addr = "";
        let commander = Commander::new(addr, self.acceptors.clone(), self.replicas.clone(), pval);
        commander::run();
       });

    }

    fn spawn_scout(&mut self, bal: BallotNum) { //change this to an actual spawn
        spawn(proc() {
            let addr = "";
            let scout = Scout::new(addr, self.acceptors.clone(), self.lu_slot_num, bal);
            scout::run();
        });


    }
}