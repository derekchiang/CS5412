pub type SlotNum = uint;
pub type Proposal = (SlotNum, Command);

#[deriving(Encodable, Decodable, Eq, Show, Clone)]
pub struct Command {
    id: ~str,
    command_name: ~str,
    args: ~[~str]
}

// impl Eq for Command {
//     fn eq(&self, that: &Command) -> bool {
//         self.id == that.id
//     }
// }

#[deriving(Encodable, Decodable, Show)]
pub enum Message {
    // client to replica
    Request(Command),

    // leader to replica
    Decision(Proposal),

    Accept,
}