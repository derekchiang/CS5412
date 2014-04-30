use common;
use common::{Command, Request};
use replica::{Replica, StateMachine};

use busybee::{Busybee, BusybeeMapper};

struct STM {
    counter: uint
}

impl StateMachine<~str> for STM {
    fn new() -> STM {
        return STM {
            counter: 1
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

            _ => {
                ~"unexpected"
            }
        }
    }
}

#[test]
fn test_replica() {
    let rid = 1u64;
    let id = 2u64;

    let replica = Replica::new::<STM, ~str>(rid, ~[id]);
    replica.run();

    let bb = Busybee::new(id, common::lookup(id), 0, BusybeeMapper::new(common::lookup));
    let msg = Request(Command{
        from: id,
        id: 1,
        name: ~"inc",
        args: vec!(~"3")
    });
    bb.send_object(rid, msg);
}