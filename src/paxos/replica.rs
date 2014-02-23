extern crate russenger;

use std::io::net::ip::SocketAddr;
use std::mem;

use common::{Command, Message, SlotNum, Proposal};
use common::{Request, Decision};

pub trait StateMachine {
    fn new() -> Self;
    fn destroy(self);
    fn clone(&self) -> Self;
    fn invoke_command(&mut self, command: Command);
}

pub struct Replica<T> {
    state: ~T,
    slot_num: SlotNum,
    proposals: ~[Proposal],
    decisions: ~[Proposal],
    addr: SocketAddr,
    leaders: ~[SocketAddr]
}

// macro_rules! with_mem_replaced(($my_lst:expr, $new_lst:ident, $ops:expr) => {{
//     let $new_lst = mem::replace(&mut $my_lst, ~[]);
//     $ops;
//     mem::replace(&mut $my_lst, $new_lst);
// }})

// This macro helps to solve the problem that, when you iterate through
// a list within a struct, you are borrowing that struct, as a result of
// which you can't call mutable function on that struct inside the loop.
// Thus, this macro swaps out the list for the loop and swaps it in at
// the end.
macro_rules! mem_iter(($obj:ident, $my_lst:expr, $ops:expr) => {{
    let lst = mem::replace(&mut $my_lst, ~[]);
    for $obj in lst.iter() {
        $ops;
    }
    mem::replace(&mut $my_lst, lst);
}})

impl<T: StateMachine> Replica<T> {
    pub fn new(addr: SocketAddr, leaders: ~[SocketAddr]) -> Replica<T> {
        Replica {
            state: ~StateMachine::new(),
            slot_num: 1u,
            proposals: ~[],
            decisions: ~[],
            addr: addr,
            leaders: leaders
        }
    }

    pub fn run(mut ~self) {
        let (port, chan) = russenger::new::<Message>(self.addr.clone());
        loop {
            let (sender, msg) = port.recv();
            match msg {
                Request(c) => self.propose(c),
                Decision((snum, comm)) => {
                    self.decisions.push((snum, comm));
                    let mut performed = false;
                    loop {
                        mem_iter!(dec, self.decisions, {
                            let (s1, p1) = dec.clone();
                            if s1 == self.slot_num {
                                mem_iter!(prop, self.proposals, {
                                    let (s2, p2) = prop.clone();
                                    if s2 == self.slot_num && p2 != p1 {
                                        self.propose(p2);
                                    }
                                });
                                self.perform(p1);
                                performed = true
                            }
                        });
                        if performed == false { break; }
                    }
                },
                other_msg => info!("Receiving a wrong message: {}", other_msg), 
            }
        }
    }

    fn propose(&mut self, comm: Command) {
        // for dec in self.decisions.iter() {
        //     let (s, p) = dec.clone();
        //     // Skip duplicated commands
        //     if p == comm {
        //         return;
        //     }
        // }
        // let prop = (self.next_slot_num, comm);
        // self.proposals.push(prop);
        // for leader in self.leaders.iter() {
        //     self.chan.send((leader, Propose(prop)));
        // }
    }

    fn perform(&mut self, comm: Command) {

    }
}