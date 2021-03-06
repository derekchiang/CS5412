#[phase(syntax, link)] extern crate log;

use std::mem;

use collections::hashmap::{HashSet, HashMap};

use common;
use common::{DataConstraint, ServerID, SlotNum, BallotNum, Command, Pvalue};
use common::{Proposal, Message, Propose, Adopted, Preempted, P1b, P2b, Terminate};

use busybee::{Busybee, BusybeeMapper};

use scout::Scout;
use commander::Commander;

pub struct Leader<X> {
    id: ServerID,
    next_sub_id: ServerID,
    ballot_num: BallotNum,
    active: bool,
    proposals: HashSet<Proposal>,
    acceptors: Vec<ServerID>,
    replicas: Vec<ServerID>,
    bb: Busybee,
    // Channels for messages to scouts and commanders
    chans: HashMap<ServerID, Sender<(ServerID, Message<X>)>>
}

impl<'a, X: DataConstraint<'a>> Leader<X> {
    pub fn new(sid: ServerID, acceptors: Vec<ServerID>, replicas: Vec<ServerID>) -> Leader<X> {
        let bb = Busybee::new(sid, common::lookup(sid), 0, BusybeeMapper::new(common::lookup));
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
            match self.bb.recv_object::<Message<X>>() {
                Ok((from, msg)) => {
                    info!("leader {}: recv {} from {}", self.id, msg, from);
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

                            self.active = true;
                        }

                        Preempted(bnum) => {
                            if bnum > self.ballot_num {
                                self.active = false;
                                let (n, _) = bnum;
                                self.ballot_num = (n + 1, self.id);
                                self.spawn_scout();
                            }
                        }

                        // Forward to scouts
                        P1b(scout_id, bnum, pvalues) => {
                            let ch = self.chans.find_copy(&scout_id).unwrap();
                            let still_open = ch.try_send((from, P1b(scout_id, bnum, pvalues)));
                            if !still_open {
                                self.chans.remove(&scout_id);
                            }
                        }

                        // Forward to commanders
                        P2b(commander_id, bnum) => {
                            let ch = self.chans.find_copy(&commander_id).unwrap();
                            let still_open = ch.try_send((from, P2b(commander_id, bnum)));
                            if !still_open {
                                self.chans.remove(&commander_id);
                            }
                        }

                        Terminate => {
                            // Terminate all scouts and commanders
                            for chan in self.chans.values() {
                                chan.send((0, Terminate));
                            }

                            return;
                        }

                        _ => error!("ERROR: wrong message {} from {}", msg, from)
                    }
                }

                Err(e) => error!("ERROR: {}", e)
            }
        }
    }

    fn spawn_scout(&mut self) {
        let (tx, rx) = channel();
        let scout = Scout::new(self.next_sub_id, self.id, self.acceptors.clone(),
            self.ballot_num, self.bb, rx);
        info!("leader {}: spawning a scout with id {}", self.id, scout.id);
        self.chans.insert(scout.id, tx);
        self.next_sub_id += 1;
        spawn(proc() {
            scout.run();
        });
    }

    fn spawn_commander(&mut self, pval: Pvalue) {
        let (tx, rx) = channel();
        let commander = Commander::new(self.next_sub_id, self.id, self.acceptors.clone(),
            self.replicas.clone(), pval, self.bb, rx);
        info!("leader {}: spawning a commander with id {}", self.id, commander.id);
        self.chans.insert(commander.id, tx);
        self.next_sub_id += 1;
        spawn(proc() {
            commander.run();
        });
    }
}