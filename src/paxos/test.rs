#![allow(unused_imports)]
#![allow(unused_variable)]
#![allow(unused_mut)]

use common;
use common::{Message, Command, Request, Response, Propose, Proposal, Decision, ServerID};
use common::{P1a, P1b, P2a, P2b};
use replica::{Replica, StateMachine};
use leader::Leader;
use acceptor::Acceptor;
use scout::Scout;
use commander::Commander;

use busybee::{Busybee, BusybeeMapper, TIMEOUT};

type Msg = Message<~str>;

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

fn expect_no_msg(mut bb: Busybee) {
    match bb.recv_object::<Msg>() {
        Err(TIMEOUT) => { /* The only valid response */ },
        x => fail!("{}", x),
    }
}

#[test]
fn test_replica() {
    use std::io::stdio::stdout;
    let mut stdout = stdout();

    let rid = 1u64 << 32;
    let id = 2u64 << 32;

    let replica = Replica::<STM, ~str>::new(rid, vec!(id));
    spawn(proc() {
        replica.run()
    });

    let mut bb = Busybee::new(id, common::lookup(id), 0, BusybeeMapper::new(common::lookup));
    bb.set_timeout(1000);
    
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
    expect_no_msg(bb);

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

    bb.shutdown();
}

#[test]
fn test_leader() {
    use std::io::stdio::stdout;
    let mut stdout = stdout();

    println!("BP1");

    let lid = 1u64 << 32;
    let id = 2u64 << 32;

    let mut bb = Busybee::new(id, common::lookup(id), 0, BusybeeMapper::new(common::lookup));
    bb.set_timeout(1000);

    let leader = Leader::<~str>::new(lid, vec!(id), vec!(id));
    spawn(proc() {
        leader.run();
    });

    // Test case 1:
    // At the beginning, the leader should spawn a scout, which sends us a P2a message.
    println!("BP2");
    let (sid, msg): (ServerID, Msg) = bb.recv_object().unwrap();
    println!("BP3");
    assert_eq!(sid, lid);
    match msg {
        P1a(scout_id, bnum) => {
            assert_eq!(scout_id, 0); // the first scout should have id 0
            assert_eq!(bnum, (0, lid)); // the first ballot
            bb.send_object::<Msg>(sid, P1b(scout_id, (0, lid), vec!()));
        },

        _ => fail!(""),
    }

    // Test case 2:
    // We shouldn't receive a P2a yet, because we haven't proposed anything
    expect_no_msg(bb);

    // Test case 3:
    // We now send a Propose just like a replica would do.  When the leader receives
    // this message, it should spawn a commander which sends a P2b to the accceptor.
    let cmd = Command{
        from: id,
        id: 1,
        name: ~"inc",
        args: vec!(~"3")
    };
    bb.send_object::<Msg>(lid, Propose((0, cmd.clone())));
    println!("BP4");

    let (sid, msg): (ServerID, Msg) = bb.recv_object().unwrap();
    println!("BP5");
    assert_eq!(sid, lid);
    match msg {
        P2a(commander_id, pval) => {
            assert_eq!(sid, lid);
            assert_eq!(commander_id, 1); // the first commander should have id 0
            let (bnum, snum, comm) = pval;
            assert_eq!(bnum, (0, lid)); // the first ballot
            assert_eq!(snum, 0);
            assert_eq!(comm, cmd);
            bb.send_object::<Msg>(sid, P2b(commander_id, bnum));
        },

        _ => fail!(""),
    }

    // Test case 4:
    // Now, the commander should decide on the proposal and sends us a response
    println!("BP6");
    let (sid, msg): (ServerID, Msg) = bb.recv_object().unwrap();
    println!("BP7");
    assert_eq!(sid, lid);
    match msg {
        Decision((snum, comm)) => {
            assert_eq!(snum, 0);
            assert_eq!(comm, cmd);
        },

        _ => fail!("wrong message: {}", msg)
    }
}