use std::fmt::Show;
use std::io::IoError;
use std::mem;

use serialize::{Encodable, Decodable};
use serialize::json::{Encoder, Decoder};
use serialize::json;

use collections::hashmap::{HashSet, HashMap};

use common;
use common::{ServerID, SlotNum, BallotNum, Command, Pvalue, Proposal, Message, Propose, Adopted, Preempted};

use busybee::{Busybee, BusybeeMapper};

use scout::Scout;
use commander::Commander;

pub struct Leader<X> {
    id: ServerID,
    next_sub_id: ServerID,
    ballot_num: BallotNum,
    active: bool,
    proposals: HashSet<Proposal>,
    acceptors: ~[ServerID],
    replicas: ~[ServerID],
    bb: Busybee,
    // Channels for messages to scouts and commanders
    chans: HashMap<ServerID, Sender<(ServerID, Message<X>)>>
}

impl<'a, X: Send + Show + Encodable<Encoder<'a>, IoError> + Decodable<Decoder, json::Error>> Leader<X> {
    pub fn new(sid: ServerID, acceptors: ~[ServerID], replicas: ~[ServerID]) -> Leader<X> {
        let bb = Busybee::new(sid, common::lookup(sid), 4, BusybeeMapper::new(common::lookup));
        Leader {
            id: sid,
            next_sub_id: 0u64,  // ids for scouts and commanders
            ballot_num: (0u64, sid),
            active: false,
            proposals: HashSet::new(),
            acceptors: acceptors,
            replicas: replicas,
            bb: bb,
            chans: HashMap::new()
        }
    }

    pub fn run(mut self) {
        self.spawn_scout();
        loop {
            let (_, msg): (ServerID, Message<X>) = self.bb.recv_object().unwrap();
            match msg {
                Propose(proposal) => {
                    if !self.proposals.contains(&proposal) {
                        self.proposals.insert(proposal.clone());
                        let (s, p) = proposal;
                        if self.active {
                            self.spawn_commander((self.ballot_num, s, p));
                        }
                    }
                }

                Adopted(bnum, pvalues) => {

                    let mut tmp: HashMap<SlotNum, (BallotNum, Command)> = HashMap::new();
                    for (b, s, p) in pvalues.move_iter() {
                        match tmp.find(&s) {
                            Some(&(old_b, _)) => {
                                if b > old_b {
                                    tmp.insert(s, (b, p));
                                }
                            }
                            None => {
                                tmp.insert(s, (b, p));
                            }
                        }
                    }

                    let mut tmp2 = HashMap::new();
                    for (s, (_, p)) in tmp.move_iter() {
                        tmp2.insert(s, p);
                    }

                    let mut proposals = HashSet::new();
                    // swap out self.proposals to avoid partial move of self
                    mem::swap(&mut proposals, &mut self.proposals);
                    for (s, p) in proposals.move_iter() {
                        if tmp2.find(&s).is_none() {
                            tmp2.insert(s, p);
                        }
                    }

                    let mut tmp3 = HashSet::new();
                    for (s, p) in tmp2.move_iter() {
                        tmp3.insert((s, p));
                    }

                    mem::swap(&mut self.proposals, &mut tmp3);

                    for (s, p) in self.proposals.clone().move_iter() {
                        self.spawn_commander((bnum, s, p));
                    }

                    // let max_pvalues = self.pmax(pvalues);
                    // self.proposals = self.p_update(&self.proposals, max_pvalues);
                    // let prop_clone = self.proposals.clone();
                    // for (s, p) in prop_clone.move_iter() {
                    //     self.spawn_commander((self.ballot_num, s, p));
                    // }
                    // self.active = true;
                }

                // Preempted((b_num, _)) => {
                //     let (curr_num, _) = self.ballot_num;
                //     if b_num > curr_num {
                //         self.active = false;
                //         self.ballot_num = (b_num + 1, self.id);
                //         self.spawn_scout(self.ballot_num);
                //     }
                // }

                _ => {} //need some debug statement here 
            }
            
        }
    }

    fn spawn_scout(&mut self) {
        let (tx, rx) = channel();
        let scout = Scout::new(self.next_sub_id, self.id, self.acceptors.clone(),
            self.ballot_num, self.bb, rx);
        self.chans.insert(self.next_sub_id, tx);
        self.next_sub_id += 1;
        spawn(proc() {
            scout.run();
        });
    }

    fn spawn_commander(&mut self, pval: Pvalue) {
        let (tx, rx) = channel();
        let commander = Commander::new(self.next_sub_id, self.id, self.acceptors.clone(),
            self.replicas.clone(), pval, self.bb, rx);
        self.chans.insert(self.next_sub_id, tx);
        self.next_sub_id += 1;
        spawn(proc() {
            commander.run();
        });
    }
}