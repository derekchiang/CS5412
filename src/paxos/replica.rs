// Some random notes:
// 1. There seems to be a lot of code that tests for the existence of certain
// elements in an array, and currently we are simply iterating the arrays.
// We should optimize the search routine. (binary search? bloom filters?)

extern crate msgpack;
extern crate serialize;
extern crate russenger;

use std::fmt::Show;
use std::io::net::ip::SocketAddr;

use serialize::{Encodable, Decodable};

use msgpack::{Encoder, Decoder};

use common::{Command, Message, SlotNum, Proposal};
use common::{Request, Decision, Propose, Response};

pub trait StateMachine<'a, T> {
    fn new() -> Self;
    fn destroy(self);
    fn clone(&self) -> Self;
    fn invoke_command(&mut self, command: Command) -> T;
}

pub struct Replica<T, X> {
    state: T,  // Would we benefit by using ~T instead?
    slot_num: SlotNum,  // specifies the slot where the next decision resides
    next_slot_num: SlotNum,  // specifies the slot which the next proposal uses
    proposals: ~[Proposal],
    decisions: ~[Proposal],
    addr: SocketAddr,
    leaders: ~[SocketAddr],
    port: Port<(SocketAddr, Message<X>)>,
    chan: Chan<(SocketAddr, Message<X>)>
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

impl<'a, T: StateMachine<'a, X>, X: Send + Show + Encodable<Encoder<'a>> + Decodable<Decoder<'a>>> Replica<T, X> {
    pub fn new(addr: SocketAddr, leaders: ~[SocketAddr]) -> Replica<T, X> {
        let (port, chan) = russenger::new::<Message<X>>(addr.clone());
        Replica {
            state: StateMachine::new(),
            slot_num: 1u,
            next_slot_num: 1u,
            proposals: ~[],
            decisions: ~[],
            addr: addr,
            leaders: leaders,
            port: port,
            chan: chan,
        }
    }

    pub fn run(mut ~self) {
        loop {
            let (_, msg) = self.port.recv();
            match msg {
                Request(c) => { self.propose(c) }
                
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

                _ => info!("Receiving a wrong message: {}", msg), 
            }
        }
    }

    fn propose(&mut self, comm: Command) {
        for dec in self.decisions.iter() {
            let (_, p) = dec.clone();
            // Skip duplicated commands
            if p == comm { return; }
        }
        let prop = (self.next_slot_num, comm);
        self.next_slot_num += 1;  // Figure out how next_slot_num works
        self.proposals.push(prop.clone());
        for leader in self.leaders.iter() {
            self.chan.send((leader.clone(), Propose(prop.clone())));
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
            self.chan.send((from_str(comm.from).unwrap(), Response(comm.id, res)));
        }

        if self.next_slot_num <= self.slot_num {
            self.next_slot_num = self.slot_num + 1;
        }
    }
}