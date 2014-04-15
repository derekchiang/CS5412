extern crate msgpack;
extern crate serialize;
extern crate russenger;

use std::fmt::Show;
use std::io::net::ip::SocketAddr;

use serialize::{Encodable, Decodable};

use msgpack::{Encoder, Decoder};

use common::{LeaderId, Proposal, BallotNum, SlotNum, Message, Pvalue, Decision, Propose, Adopted, Preempted};
use common::{P1a, P1b, P2a, P2b};

pub struct Scout<X> {
    acceptors: ~[SocketAddr],
    lu_slot_num: SlotNum,
    bal: BallotNum,
    pvalues: HashSet<Pvalue>,
    tx: Sender<(SocketAddr, Message<X>)>,
    rx: Receiver<(SocketAddr, Message<X>)>,
    inner_tx: Sender<Message<X>>,
    inner_rx: Receiver<Message<X>>,
}

impl<'a, X: Send + Show + Encodable<Encoder<'a>> + Decodable<Decoder<'a>>> Scout<X> {

    pub fn new(addr: SocketAddr, acceptors: ~[SocketAddr], slt: SlotNum, bal: BallotNum) -> Scout<X> {
        let (tx, rx) = russenger::new::<Message<X>>(addr.clone());
        let (inner_tx, inner_rx) = channel();
        Scout {
            bal: bal,
            lu_slot_num: slt,
            acceptors: acceptors,
            tx: tx,
            rx: rx,
            inner_tx: inner_tx,
            inner_rx: inner_rx,
            pvalues: HashSet::new(),
        }
    }

    pub fn run(~mut self) {
        let mut waiting_for = self.acceptors.clone();
        let acc_clone = self.acceptors.clone();
        for acc in acc_clone.iter() {
            self.tx.send((acc.clone(), P1a(self.bal, self.lu_slot_num))); //hopefully correct parameters for the P1a
        }
        loop {
            let (acc, msg) = self.rx.recv(); //probably wrong
            match msg {
                P1b((b_num, l_id), pvals) => {
                    let (my_b, _) = self.bal;
                    if b_num == my_b {
                        for pv in pvals.move_iter() {
                            pvalues.insert(pv);
                        }
                        waiting_for.remove(acc);
                        if waiting_for.iter().len() < acc_clone.iter().len() / 2 { //fix
                            self.inner_tx.send(Adopted((b_num, l_id), pvalues));
                            break;
                        }
                    } else {
                        self.inner_tx.send(Preempted((b_num, l_id))); //tell leader that this one has been preempted
                        break;
                    }

                }

                _ => {} //need some debug statement here
            }


        }


    }
}