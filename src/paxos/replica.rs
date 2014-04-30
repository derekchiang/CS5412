// Some random notes:
// 1. There seems to be a lot of code that tests for the existence of certain
// elements in an array, and currently we are simply iterating the arrays.
// We should optimize the search routine. (binary search? bloom filters?)
use std::fmt::Show;
use std::io::IoError;

use serialize::{Encodable, Decodable};
use serialize::json::{Encoder, Decoder};
use serialize::json;

use common;
use common::{ServerID, Command, Message, SlotNum, Proposal};
use common::{Request, Decision, Propose, Response};

use busybee::{Busybee, BusybeeMapper};

pub trait StateMachine<T> {
    fn new() -> Self;
    fn destroy(self);
    fn clone(&self) -> Self;
    fn invoke_command(&mut self, command: Command) -> T;
}

pub struct Replica<T, X> {
    state: T,  // Would we benefit by using ~T instead?
    slot_num: SlotNum,  // specifies the slot where the next decision resides
    lowest_unused_slot_num: SlotNum,  // specifies the slot which the next proposal uses
    proposals: ~[Proposal],
    decisions: ~[Proposal],
    leaders: ~[ServerID],
    bb: Busybee
}

// This macro helps to solve the problem that, when you iterate through
// a list within a struct, you are borrowing that struct, as a result of
// which you can't call mutable function on that struct inside the loop.
// Thus, this macro swaps out the list for the loop and swaps it in at
// the end.
// 
// TODO: doesn't seem like we need it anymore?
// 
// macro_rules! mem_iter(($obj:ident, $my_lst:expr, $ops:expr) => {{
//     let lst = mem::replace(&mut $my_lst, ~[]);
//     for $obj in lst.iter() {
//         $ops;
//     }
//     mem::replace(&mut $my_lst, lst);
// }})

impl<'a, T: StateMachine<X>, X: Send + Show + Encodable<Encoder<'a>, IoError> + Decodable<Decoder, json::Error>> Replica<T, X> {
    pub fn new(sid: ServerID, leaders: ~[ServerID]) -> Replica<T, X> {
        let bb = Busybee::new(sid, common::lookup(sid), 0, BusybeeMapper::new(common::lookup));
        Replica {
            state: StateMachine::new(),
            slot_num: 0u64,
            lowest_unused_slot_num: 0u64,
            proposals: ~[],
            decisions: ~[],
            leaders: leaders,
            bb: bb
        }
    }

    pub fn run(mut self) {
        loop {
            let (_, msg): (ServerID, Message<X>) = self.bb.recv_object().unwrap();
            // TODO: we should verify that:
            // 1. If the message is a Request, then the sender ID matches the id field of the Request.
            // 2. If the message is a Decision, then the sender should indeed be a leader.
            match msg {
                Request(c) => {
                    self.propose(c);
                }
                
                Decision((snum, comm)) => {
                    self.decisions.push((snum, comm));
                    let mut performed = false;
                    loop {
                        let mut to_perform = ~[];
                        let mut to_propose = ~[];
                        for dec in self.decisions.iter() {
                            let (s1, p1) = dec.clone();
                            if s1 == self.slot_num {
                                for prop in self.proposals.iter() {
                                    let (s2, p2) = prop.clone();
                                    if s2 == self.slot_num && p2 != p1 {
                                        to_propose.push(p2);
                                    }
                                }
                                to_perform.push(p1);
                                performed = true
                            }
                        }
                        to_propose.move_iter().map(|comm| self.propose(comm) );
                        to_perform.move_iter().map(|comm| self.perform(comm) );
                        if performed == false { break; }
                    }
                }
                _ => {} //need some debug statement here
//                _ => info!("Receiving a wrong message: {}", msg), 
            }
        }
    }

    fn propose(&mut self, comm: Command) {
        for dec in self.decisions.iter() {
            let (_, p) = dec.clone();
            // Skip duplicated commands
            if p == comm { return; }
        }
        let prop = (self.lowest_unused_slot_num, comm);
        self.lowest_unused_slot_num += 1;
        self.proposals.push(prop.clone());
        for leader in self.leaders.iter() {
            self.bb.send_object::<Message<X>>(leader.clone(), Propose(prop.clone()));
        }
    }

    fn perform(&mut self, comm: Command) {
        let mut found = false;
        for dec in self.decisions.iter() {
            let (_, c) = dec.clone(); 
            if c == comm { found = true; }
        }

        if found {
            self.slot_num += 1;
        } else {
            let res = self.state.invoke_command(comm.clone());
            self.slot_num += 1;
            self.bb.send_object(comm.from, Response(comm.id, res));
        }

        if self.lowest_unused_slot_num < self.slot_num {
            self.lowest_unused_slot_num = self.slot_num;
        }
    }
}