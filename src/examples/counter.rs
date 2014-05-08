extern crate paxos;

use paxos::StateMachine;
use paxos::Command;

struct STM {
    counter: uint
}

impl StateMachine<~str> for STM {
    fn new() -> STM {
        return STM {
            counter: 0
        }
    }

    fn destroy(self) {}

    fn clone(&self) -> STM {
        STM {
            counter: self.counter
        }
    }

    fn invoke_command(&mut self, command: Command) -> ~str {
        let mut command = command;
        match command.name.as_slice() {
            "inc" => {
                self.counter += from_str(command.args.remove(0).unwrap()).unwrap();
                ~"ok"
            }

            "dec" => {
                self.counter -= from_str(command.args.remove(0).unwrap()).unwrap();
                ~"ok"
            }

            "read" => {
                format!("{}", self.counter)
            }

            _ => {
                ~"unexpected"
            }
        }
    }
}

fn main() {
    let mut client = paxos::create_cluster::<~str, STM>();
    for _ in range(0, 10) {
        println!("Reply for inc: {}", client.call(~"inc", vec!(~"1")).recv());
    }

    println!("Reply for read: {}", client.call(~"read", vec!()).recv());
}