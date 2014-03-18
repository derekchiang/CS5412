extern crate msgpack;
extern crate serialize;

pub type SlotNum = uint;
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

#[deriving(Encodable, Decodable, Show, Clone)]
pub struct Command {
    // This really should be a SocketAddr, but annoyingly SocketAddr is
    // neither encodable nor decodable, so we resort to using a str and
    // convert it to/from SocketAddr as needed.
    from: ~str,
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
}