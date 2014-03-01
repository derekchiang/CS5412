extern crate msgpack;
extern crate serialize;
extern crate russenger;

use std::fmt::Show;
use std::io::net::ip::SocketAddr;

use serialize::{Encodable, Decodable};

use msgpack::{Encoder, Decoder};

use common::{BallotNum, Pvalue, Message};
use common::{P1a, P1b, P2a, P2b};

pub struct Acceptor<X> {
    ballot_num: BallotNum,
    accepted: ~[Pvalue],
    port: Port<(SocketAddr, Message<X>)>,
    chan: Chan<(SocketAddr, Message<X>)>,
}

impl<'a, X: Send + Show + Encodable<Encoder<'a>> + Decodable<Decoder<'a>>> Acceptor<X> {
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
        loop {
            let (leader, msg) = self.port.recv();
            match msg {
                P1a(bnum, snum) => {
                    if bnum > self.ballot_num {
                        self.ballot_num = bnum;
                    }

                    let pvalues_to_respond = self.accepted.iter().filter_map(|pvalue| {
                        let &(_, slot_num, _) = pvalue;
                        if slot_num >= snum { Some(pvalue.clone()) } else { None }
                    }).collect();

                    self.chan.send((leader, P1b(self.ballot_num, pvalues_to_respond)));
                }

                P2a(pvalue) => {
                    let (b, _, _) = pvalue.clone();
                    if b >= self.ballot_num {
                        self.ballot_num = b;
                        self.accepted.push(pvalue);
                    }
                    self.chan.send((leader, P2b(self.ballot_num)));
                }

                _ => info!("Receiving a wrong message: {}", msg)
            }
        }
    }
}