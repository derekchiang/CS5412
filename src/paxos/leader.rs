extern crate msgpack;
extern crate serialize;
extern crate russenger;

use std::fmt::Show;
use std::io::net::ip::SocketAddr;
use std::rand::random;

use serialize::{Encodable, Decodable};

use msgpack::{Encoder, Decoder};

use common::{LeaderId, Proposal, BallotNum, SlotNum, Message};

pub struct Leader<X> {
    id: LeaderId,
    ballot_num: BallotNum,
    active: bool,
    lu_slot_num: SlotNum, // lowest undecided slot number
    proposals: ~[Proposal],
    port: Port<(SocketAddr, Message<X>)>,
    chan: Chan<(SocketAddr, Message<X>)>,
}

impl<'a, X: Send + Show + Encodable<Encoder<'a>> + Decodable<Decoder<'a>>> Leader<X> {
    pub fn new(addr: SocketAddr) -> Leader<X> {
        let (port, chan) = russenger::new::<Message<X>>(addr.clone());
        let rand_id = random();  // TODO: leader IDs could collide.  This would cause problem.
        Leader {
            id: rand_id,
            ballot_num: (0u, rand_id),
            active: false,
            lu_slot_num: 0,
            proposals: ~[],
            port: port,
            chan: chan,
        }
    }

    pub fn run(mut ~self) {
        loop {

        }
    }
}