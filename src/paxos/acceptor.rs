extern crate msgpack;
extern crate serialize;
extern crate russenger;

use std::io::net::ip::SocketAddr;

use serialize::{Encodable, Decodable};

use msgpack::{Encoder, Decoder};

use common::{BallotNum, Pvalue, Message};

pub struct Acceptor<X> {
    ballot_num: BallotNum,
    accepted: ~[Pvalue],
    port: Port<(SocketAddr, Message<X>)>,
    chan: Chan<(SocketAddr, Message<X>)>,
}

impl<'a, X: Send + Encodable<Encoder<'a>> + Decodable<Decoder<'a>>> Acceptor<X> {
    pub fn new(addr: SocketAddr) -> Acceptor<X> {
        let (port, chan) = russenger::new::<Message<X>>(addr.clone());
        Acceptor {
            ballot_num: (0u, 0u),
            accepted: ~[],
            port: port,
            chan: chan,
        }
    }

    pub fn run(mut ~self) {

    }
}