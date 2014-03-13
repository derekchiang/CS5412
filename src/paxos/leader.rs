extern crate msgpack;
extern crate serialize;
extern crate russenger;
extern crate rand;

use std::fmt::Show;
use std::io::net::ip::SocketAddr;
use rand::random;

use serialize::{Encodable, Decodable};

use msgpack::{Encoder, Decoder};

use common::{LeaderId, Proposal, BallotNum, SlotNum, Message};

pub struct Leader<X> {
    id: LeaderId,
    ballot_num: BallotNum,
    active: bool,
    lu_slot_num: SlotNum, // lowest undecided slot number
    proposals: ~[Proposal],
    acceptors: ~[SocketAddr],
    // Communication endpoints with the outside world (acceptors, replicas)
    port: Port<(SocketAddr, Message<X>)>,
    chan: Chan<(SocketAddr, Message<X>)>,
    // Communication endpoints with the spawned commanders and scouts
    inner_port: Port<Message<X>>,
    inner_chan: Chan<Message<X>>,
}

impl<'a, X: Send + Show + Encodable<Encoder<'a>> + Decodable<Decoder<'a>>> Leader<X> {
    pub fn new(addr: SocketAddr, acceptors: ~[SocketAddr]) -> Leader<X> {
        let (port, chan) = russenger::new::<Message<X>>(addr.clone());
        let (inner_port, inner_chan) = Chan::new();
        let rand_id = random();  // TODO: leader IDs could collide.  This would cause problem.
        Leader {
            id: rand_id,
            ballot_num: (0u, rand_id),
            active: false,
            lu_slot_num: 0,
            proposals: ~[],
            acceptors: acceptors,
            port: port,
            chan: chan,
            inner_port: inner_port,
            inner_chan: inner_chan,
        }
    }

    pub fn run(mut ~self) {
        // self.spawn_scout();
        loop {
            let msg = self.inner_port.recv();
        }
    }
}