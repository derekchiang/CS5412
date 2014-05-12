extern crate paxos;
extern crate time;

use std::io::timer::sleep;
use time::precise_time_ns;

use paxos::StateMachine;
use paxos::Command;

struct Counter {
    counter: int,
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
               res_tx.send(format!("{}, {}", self.counter, precise_time_ns()));
            }

            "dec" => {
                self.counter -= from_str(comm.args.remove(0).unwrap()).unwrap();
                res_tx.send(format!("{}, {}", self.counter, precise_time_ns()));
            }

            "read" => {
                res_tx.send(format!("{}, {}", self.counter, precise_time_ns()));
            }

            _ => {
               res_tx.send(~"unexpected");
            }
        }
    }
}

fn main() {
    let cluster = paxos::new_cluster::<~str, Counter>();
    
    let client1 = cluster.new_client(7u64 << 32);
    let client2 = cluster.new_client(8u64 << 32);
    let mut client3 = cluster.new_client(9u64 << 32);

    spawn(proc() {
        let mut client1 = client1;
        for _ in range(0, 10) {
            let res = client1.call(~"inc", vec!(~"1")).recv();
            println!("Client 1: {}", res);
        }
    });

    spawn(proc() {
        let mut client2 = client2;
        for _ in range(0, 10) {
            let res = client2.call(~"dec", vec!(~"1")).recv();
            println!("Client 2: {}", res);
        }
    });

    sleep(1000);
    println!("Finally: {}", client3.call(~"read", vec!()).recv());
}