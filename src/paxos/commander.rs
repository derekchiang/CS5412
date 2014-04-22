// extern crate msgpack;
// extern crate serialize;
// extern crate russenger;

// use std::fmt::Show;
// use std::io::net::ip::SocketAddr;

// use serialize::{Encodable, Decodable};

// use msgpack::{Encoder, Decoder};

// use common::{LeaderId, Proposal, BallotNum, SlotNum, Message, Pvalue, Decision, Propose, Adopted, Preempted};
// use common::{P1a, P1b, P2a, P2b};

// pub struct Commander<X> {
// 	acceptors: ~[SocketAddr],
// 	replicas: ~[SocketAddr],
// 	pval: Pvalue,
// 	tx: Sender<(SocketAddr, Message<X>)>,
//     rx: Receiver<(SocketAddr, Message<X>)>,
//     inner_tx: Sender<Message<X>>,
//     inner_rx: Receiver<Message<X>>,
// }

// impl<'a, X: Send + Show + Encodable<Encoder<'a>> + Decodable<Decoder<'a>>> Commander<X> {
// 	pub fn new(addr: SocketAddr, acceptors: ~[SocketAddr], replicas: ~[SocketAddr], pval: Pvalue) {
// 		let (tx, rx) = russenger::new::<Message<X>>(addr.clone());
//         let (inner_tx, inner_rx) = channel();
//         Leader {
//         	acceptors: acceptors;
//         	replicas: replicas;
//         	pval: pval;
//         	tx: tx,
//         	rx: rx,
//         	inner_tx: inner_tx,
//         	inner_rx: inner_rx,
//         }
// 	}

// 	pub fn run(mut ~self) {
// 		let acc_clone = self.acceptors.clone();
//         let rep_clone = self.replicas.clone();
//         let mut waiting_for = acc_clone.clone();
//         for acc in acc_clone.iter() {
//             self.tx.send((acc.clone(), P2a(self.pval.clone())));
//         }
//         loop {
//             let (acc, msg) = self.rx.recv(); //somehow?
//             match msg {
//                 P2b((b_num, l_id)) => {
//                     let ((bal, _), slt, ref comm) = pval; //make sure this doesnt cause problems with the ownerships
//                     if b_num == bal {
//                         waiting_for.remove(acc);
//                         if waiting_for.iter().len() < acc_clone.iter().len() / 2 { //fix
//                             for rep in rep_clone.iter() {
//                                 self.tx.send((rep.clone(), Decision((slt, comm.clone()))));
//                             }
//                             break;
//                         }
//                     } else {
//                         self.inner_tx.send(Preempted((b_num, l_id))); //tell leader that this one has been preempted
//                         break;
//                     }
//                 }

//                 _ => {} //need some debug statement here
//             }
// 	}
// }