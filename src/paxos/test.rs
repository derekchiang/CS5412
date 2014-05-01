use common;
use common::{Message, Command, Request, Response, Propose, Proposal, Decision, ServerID};
use replica::{Replica, StateMachine};

use busybee::{Busybee, BusybeeMapper, TIMEOUT};

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

#[test]
fn test_replica() {
    use std::io::stdio::stdout;
    let mut stdout = stdout();
    type Msg = Message<~str>;

    let rid = 1u64 << 32;
    let id = 2u64 << 32;

    let replica = Replica::<STM, ~str>::new(rid, ~[id]);
    spawn(proc() {
        replica.run()
    });

    let mut bb = Busybee::new(id, common::lookup(id), 0, BusybeeMapper::new(common::lookup));
    
    // Test case 1:
    // When a Request is sent to the replica, the replica should send a
    // Propose to the leader.
    let cmd = Command{
        from: id,
        id: 1,
        name: ~"inc",
        args: vec!(~"3")
    };
    let msg = Request(cmd.clone());

    bb.send_object::<Msg>(rid, msg);

    let (sid, msg): (ServerID, Msg) = bb.recv_object().unwrap();
    assert_eq!(sid, rid);
    match msg {
        Propose((slot_num, comm)) => {
            // Since this is the first request the replica received,
            // the slot number should be 0.
            assert_eq!(slot_num, 0);
            assert_eq!(comm, cmd);
        },

        _ => fail!("wrong message: {}", msg)
    }

    // Test case 2:
    // When a Decision is sent to the replica, the replica should perform the
    // decision and return the result to the client who sent the original request.
    let msg = Decision((0, cmd.clone()));

    bb.send_object::<Msg>(rid, msg);

    let (sid, msg): (ServerID, Msg) = bb.recv_object().unwrap();
    assert_eq!(sid, rid);
    match msg {
        Response(comm_id, res) => {
            assert_eq!(comm_id, cmd.id);
            assert_eq!(res, ~"ok");
        },

        _ => fail!("wrong message: {}", msg)
    }

    // Test case 3:
    // Since a decision has already been made for the 0th slot, another
    // decision should have no effect on the replica.
    let msg = Decision((0, cmd.clone()));

    bb.send_object::<Msg>(rid, msg);
    bb.set_timeout(1000);

    match bb.recv_object::<Msg>() {
        Err(TIMEOUT) => { /* The only valid response */ },
        x => fail!("{}", x),
    }
    bb.set_timeout(-1);

    // Test case 4:
    // Now, let's make sure the counter was incremented successfully.
    let cmd = Command{
        from: id,
        id: 2,
        name: ~"read",
        args: vec!()
    };
    let msg = Request(cmd.clone());

    bb.send_object::<Msg>(rid, msg);

    let (sid, msg): (ServerID, Msg) = bb.recv_object().unwrap();
    assert_eq!(sid, rid);
    match msg {
        Propose((slot_num, comm)) => {
            assert_eq!(slot_num, 1);
            assert_eq!(comm, cmd);
        },

        _ => fail!("wrong message: {}", msg)
    }

    let msg = Decision((1, cmd.clone()));

    bb.send_object::<Msg>(rid, msg);

    let (sid, msg): (ServerID, Msg) = bb.recv_object().unwrap();
    assert_eq!(sid, rid);
    match msg {
        Response(comm_id, res) => {
            assert_eq!(comm_id, cmd.id);
            assert_eq!(res, ~"3");
        },

        _ => fail!("wrong message: {}", msg)
    }
}