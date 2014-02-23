#[crate_id = "paxos#0.1"];
#[comment = "An implementation of the multi-Paxos consensus protocol."];
#[license = "MIT/ASL2"];
#[crate_type = "lib"];

#[feature(macro_rules)];

extern crate serialize;
extern crate uuid;

pub use replica::StateMachine;

mod common;
mod replica;

#[cfg(test)]
mod tests {
    extern crate russenger;
    
    #[test]
    fn do_something() {
        println!("Hello World!");
    }
}