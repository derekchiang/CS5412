use std::io;
use std::io::fs::File;
use std::io::MemReader;
use std::io::net::ip::SocketAddr;
use std::io::IoError;
use std::fmt::Show;
use std::path::Path;

use serialize::json;
use serialize::{Encodable, Decodable};
use serialize::json::{Encoder, Decoder};

pub trait DataConstraint<'a>: Share + Send + Show + Encodable<Encoder<'a>, IoError> + Decodable<Decoder, json::Error> {}
impl<'a, T: Share + Send + Show + Encodable<Encoder<'a>, IoError> + Decodable<Decoder, json::Error>> DataConstraint<'a> for T {}

pub trait StateMachine<T>: Send {
    fn new() -> Self;
    fn destroy(self);
    fn clone(&self) -> Self;
    fn invoke_command(&mut self, res_tx: Sender<T>, comm: Command);
}

pub type ServerID = u64;
pub type CommandID = u64;
pub type SlotNum = u64;
#[deriving(Hash, Show)]
pub type Proposal = (SlotNum, Command);
#[deriving(TotalOrd)]
pub type BallotNum = (u64, ServerID);
pub type Pvalue = (BallotNum, SlotNum, Command);

#[deriving(Encodable, Decodable, Show, Clone, Hash, TotalEq)]
pub struct Command {
    from: ServerID,
    id: CommandID,
    name: ~str,
    args: Vec<~str>
}

impl Eq for Command {
    fn eq(&self, that: &Command) -> bool {
        self.from == that.from && self.id == that.id
    }
}

#[deriving(Encodable, Decodable, Show)]
pub enum Message<T> {
    // client to replica
    Request(Command),

    // leader to replica
    Decision(Proposal),

    Propose(Proposal),

    Response(u64, T),

    P1a(ServerID, BallotNum),

    P1b(ServerID, BallotNum, Vec<Pvalue>),

    P2a(ServerID, Pvalue),

    P2b(ServerID, BallotNum),

    Adopted(BallotNum, Vec<Pvalue>), //scout to leader

    Preempted(BallotNum), //scout or commander to leader

    Terminate
}

pub fn lookup(server_id: ServerID) -> SocketAddr {
    #[deriving(Decodable)]
    struct Server {
        id: u64,
        addr: ~str,
    }

    let path = Path::new("addrs.json");
    let mut file = File::open(&path);
    let content = file.read_to_end().unwrap();
    let mut content_reader = MemReader::new(content);
    let json_object = json::from_reader(&mut content_reader as &mut io::Reader).unwrap();
    let mut decoder = json::Decoder::new(json_object);
    let servers: ~[Server] = match Decodable::decode(&mut decoder) {
        Ok(v) => v,
        Err(e) => fail!("Decoding error: {}", e)
    };

    for s in servers.move_iter() {
        if s.id == server_id >> 32 {
            return from_str::<SocketAddr>(s.addr).unwrap();
        }
    }
    fail!("Invalid server id: {}", server_id);
}
