use std::fmt::Show;
use std::io::IoError;

use serialize::json;
use serialize::{Encodable, Decodable};
use serialize::json::{Encoder, Decoder};

use common;
use common::{ServerID, BallotNum, Pvalue, Message};
use common::{P1a, P1b, P2a, P2b};

use busybee::{Busybee, BusybeeMapper};

pub struct Acceptor<X> {
    ballot_num: BallotNum,
    accepted: ~[Pvalue],
    bb: Busybee,
}

impl<'a, X: Send + Show + Encodable<Encoder<'a>, IoError> + Decodable<Decoder, json::Error>> Acceptor<X> {
    pub fn new(sid: ServerID) -> Acceptor<X> {
        let bb = Busybee::new(sid, common::lookup(sid), 1, BusybeeMapper::new(common::lookup));
        Acceptor {
            ballot_num: (0u64, 0u64),
            accepted: ~[],
            bb: bb,
        }
    }

    pub fn run(mut ~self) {
        loop {
            let (leader, msg): (ServerID, Message<X>) = self.bb.recv_object().unwrap();
            match msg {
                P1a(bnum) => {
                    if bnum > self.ballot_num {
                        self.ballot_num = bnum;
                    }

                    self.bb.send_object::<Message<X>>(leader, P1b(self.ballot_num, self.accepted.clone()));
                }

                P2a(pvalue) => {
                    let (b, _, _) = pvalue.clone();
                    if b >= self.ballot_num {
                        self.ballot_num = b;
                        self.accepted.push(pvalue);
                    }
                    self.bb.send_object::<Message<X>>(leader, P2b(self.ballot_num));
                }

//                _ => info!("Receiving a wrong message: {}", msg)
                _ => {} //need some debug statement here
            }
        }
    }
}