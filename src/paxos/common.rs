use std::io;
use std::io::fs::File;
use std::io::MemReader;
use std::io::net::ip::SocketAddr;
use std::path::Path;

use serialize::json;
use serialize::{Encodable, Decodable};

use busybee::BusybeeMapper;

pub type ServerID = u64;
pub type SlotNum = uint;
#[deriving(Hash)]
pub type Proposal = (SlotNum, Command);
pub type LeaderId = uint;
#[deriving(TotalOrd)]
pub type BallotNum = (uint, LeaderId);
pub type Pvalue = (BallotNum, SlotNum, Command);

// impl BallotNum {
//     pub fn increment(self) -> BallotNum {
//         let (n, lid) = self;
//         (n+1, lid)
//     }
// }

#[deriving(Encodable, Decodable, Show, Clone, Hash)]
pub struct Command {
    from: u64,
    id: ~str,
    command_name: ~str,
    args: ~[~str]
}

impl Eq for Command {
    fn eq(&self, that: &Command) -> bool {
        self.id == that.id
    }
}

#[deriving(Encodable, Decodable, Show)]
pub enum Message<T> {
    // client to replica
    Request(Command),

    // leader to replica
    Decision(Proposal),

    Propose(Proposal),

    Response(~str, T),

    P1a(BallotNum, SlotNum), //why does this need a slotnum?

    P1b(BallotNum, ~[Pvalue]),

    P2a(Pvalue),

    P2b(BallotNum),

    Adopted(BallotNum, ~[Pvalue]), //scout to leader

    Preempted(BallotNum), //scout or commander to leader
}

pub fn lookup(server_id: ServerID) -> SocketAddr {
    #[deriving(Decodable)]
    struct Server {
        id: u64,
        role: ~str,
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
        if s.id == server_id {
            return from_str::<SocketAddr>(s.addr).unwrap();
        }
    }
    fail!("Invalid server id: {}", server_id);
}

// static mapper: BusybeeMapper = BusybeeMapper::new(lookup);