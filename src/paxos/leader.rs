use std::fmt::Show;
use std::io::IoError;

use serialize::{Encodable, Decodable};
use serialize::json::{Encoder, Decoder};
use serialize::json;

use collections::hashmap::HashMap;

use common;
use common::{ServerID, BallotNum, Proposal, Message};

use busybee::{Busybee, BusybeeMapper};

use scout::Scout;

pub struct Leader<X> {
    id: ServerID,
    next_scout_id: ServerID,
    ballot_num: BallotNum,
    active: bool,
    proposals: ~[Proposal],
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
            next_scout_id: 0,
            ballot_num: (0u64, sid),
            active: false,
            proposals: ~[],
            acceptors: acceptors,
            replicas: replicas,
            bb: bb,
            chans: HashMap::new()
        }
    }

    pub fn run(mut ~self) {
        self.spawn_scout();
        // loop {
        //     let msg = self.inner_rx.recv();
        //     match msg {
        //         Propose((s_num, comm)) => {
        //             if !(self.chk_contains_slot(&self.proposals, s_num)) { //if proposals does not contain this slot number already
        //                 self.proposals.push((s_num, comm.clone()));
        //                 if self.active {
        //                     spawn(self.spawn_commander((self.ballot_num, s_num, comm)));
        //                    /* spawn(proc() {
        //                         let commander = Commander::new();
        //                         commander.run();
        //                     }); */
        //                 }
        //             }
        //         }
        //         Adopted(b_num, pvalues) => { //maybe check if this ballot number is the right one
        //             let max_pvalues = self.pmax(pvalues);
        //             self.proposals = self.p_update(&self.proposals, max_pvalues); // need to find out how to resolve this
        //             let prop_clone = self.proposals.clone();
        //             for (s, p) in prop_clone.move_iter() {
        //                 self.spawn_commander((self.ballot_num, s, p));
        //             }
        //             self.active = true;
        //         }
        //         Preempted((b_num, _)) => {
        //             let (curr_num, _) = self.ballot_num;
        //             if b_num > curr_num {
        //                 self.active = false;
        //                 self.ballot_num = (b_num + 1, self.id);
        //                 self.spawn_scout(self.ballot_num);
        //             }
        //         }
        //         _ => {} //need some debug statement here 
        //     }
            
        // }
    }

    fn spawn_scout(&mut self) {
        let (tx, rx) = channel();
        let scout = Scout::new(self.next_scout_id, self.id, self.acceptors.clone(), self.ballot_num, self.bb, rx);
        self.chans.insert(self.next_scout_id, tx);
        self.next_scout_id += 1;
        spawn(proc() {
            scout.run();
        });
    }
}