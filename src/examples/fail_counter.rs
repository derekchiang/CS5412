extern crate paxos;

use paxos::StateMachine;
use paxos::Command;

struct Counter {
    counter: uint
}

impl StateMachine<~str> for Counter {
    fn new() -> Counter {
        return Counter {
            counter: 0
        }
    }

    fn destroy(self) {}

    fn clone(&self) -> Counter {
        Counter {
            counter: self.counter
        }
    }

    fn invoke_command(&mut self, res_tx: Sender<~str>, comm: Command) {
        let mut comm = comm;
        match comm.name.as_slice() {
            "inc" => {
                self.counter += from_str(comm.args.remove(0).unwrap()).unwrap();
               res_tx.send(~"ok");
            }

            "dec" => {
                self.counter -= from_str(comm.args.remove(0).unwrap()).unwrap();
                res_tx.send(~"ok");
            }

            "read" => {
                res_tx.send(format!("{}", self.counter));
            }

            _ => {
               res_tx.send(~"unexpected");
            }
        }
    }
}

fn main() {
    let cluster = paxos::new_cluster::<~str, Counter>();
    let mut client = cluster.new_client(7u64 << 32);
    
    for i in range(0, 10) {
        client.call(~"inc", vec!(~"1")).recv();
        if i == 5 {
            client.terminate_acceptors(1);
        }
    }
    println!("Reply for read: {}", client.call(~"read", vec!()).recv());

    for i in range(0, 10) {
        client.call(~"dec", vec!(~"1")).recv();
        if i == 5 {
            client.terminate_replicas(1);
        }
    }
    println!("Reply for read: {}", client.call(~"read", vec!()).recv());
}