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

use acceptor::{Acceptor};
use common::{P1a, P1b, P2a, P2b, BallotNum, Message};

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
fn test_acceptor() {
    use std::io::stdio::stdout;
    let mut stdout = stdout();

    let aid = 1u64 << 32;
    let id = 2u64 << 32;
    let acc = Acceptor::<~str>::new(aid);

    spawn(proc() {
        acc.run();
    });

    let mut bb = Busybee::new(id, common::lookup(id), 0, BusybeeMapper::new(common::lookup));
    //Test Case 1:
    //send p1a message to acceptor, check for correct p2b reply
    //the accepted set should be empty, ballotnum should be 0


    //sending P1a message with id = tester's id and ballotnum = 1, this server's id (?)
    //TODO: decide what that second id needs to be
    let msg = P1a(id, (1, id));
    bb.send_object::<Msg>(aid, msg);

    let (sid, msg): (ServerID, Msg) = bb.recv_object().unwrap();
    //make sure you got a reply from the place you sent to
    assert_eq!(sid, aid);

    match msg {
        P1b(sid, (b_num, sid2), accepted) => {
            //this is the 'leader' that sent the P1a
            assert_eq!(sid, id);
            //should have the same ballot number as the one you sent
            assert_eq!(b_num, 1);
            //should have your id as the id of the server that sent that ballot
            assert_eq!(sid2, id);
            //should not have accepted anything yet
            assert_eq!(accepted, vec!());
        }
        _ => fail!("wrong message: {}", msg)
    }

    //Test Case 2:
    //send p1a message with lower ballot number to acceptor. hopefully it returns the higher one
    //pretend it was sent from mysterious server 3
    let msg = P1a(id, (0, 3u64 << 32));
    bb.send_object::<Msg>(aid, msg);

    let (sid, msg): (ServerID, Msg) = bb.recv_object().unwrap();
    //make sure you got a reply from the place you sent to
    assert_eq!(sid, aid);

    match msg {
        P1b(sid, (b_num, sid2), accepted) => {
            //this is the 'leader' that sent the P1a
            assert_eq!(sid, id);
            //should have the higher ballot number rather than the one you sent
            assert_eq!(b_num, 1);
            //should have your id as the id of the server that sent that ballot (not the mysterious server 3)
            assert_eq!(sid2, id);
            //should not have accepted anything yet
            assert_eq!(accepted, vec!());
        }
        _ => fail!("wrong message: {}", msg)
    }

    //Test Case 3:
    //send p2a message (acceptor should accept it and send the proper response)
    //then send another p1a message (acceptor should adopt it and send back accepted list containing previous pvalue)
    let cmd =  Command{
        from: id,
        id: 1,
        name: ~"echo",
        args: vec!(~"hello world")
    };
    //ballotnum = 2, slotnum = 0 (?)
    let msg = P2a(id, ((2, id), 0, cmd.clone()));
    bb.send_object::<Msg>(aid, msg);

    let (sid, msg): (ServerID, Msg) = bb.recv_object().unwrap();
    //make sure you got a reply from the place you sent to
    assert_eq!(sid, aid);

    match msg {
        P2b(sid, (b_num, sid2)) => {
            //leader that sent the P2a is me
            assert_eq!(sid, id);
            //the highest ballot num so far is 2
            assert_eq!(b_num, 2);
            //the leader that sent that ballot num is me
            assert_eq!(sid2, id);
        }
        _ => fail!("wrong message: {}", msg)
    }
    //send another P1a message to see if the command was accepted
    let msg = P1a(id, (0, id));
    bb.send_object::<Msg>(aid, msg);

    let (sid, msg): (ServerID, Msg) = bb.recv_object().unwrap();
    //make sure you got a reply from the place you sent to
    assert_eq!(sid, aid);

    match msg {
        P1b(sid, (b_num, sid2), accepted) => {
            //this is the 'leader' that sent the P1a
            assert_eq!(sid, id);
            //should have the higher ballot number rather than the one you sent
            assert_eq!(b_num, 2);
            //should have your id as the id of the server that sent that ballot (not the mysterious server 3)
            assert_eq!(sid2, id);
            //should have the previous command accepted
            assert_eq!(accepted, vec!(((2, id), 0, cmd.clone())));
        }
        _ => fail!("wrong message: {}", msg)
    }

}

//stdout.write_line('hello')
#[test]
fn test_replica() {
    use std::io::stdio::stdout;
    let mut stdout = stdout();
    type Msg = Message<~str>;
    //shift id left 32
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