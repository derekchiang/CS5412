#[crate_id = "paxos#0.1"];
#[comment = "An implementation of the multi-Paxos consensus protocol."];
#[license = "MIT/ASL2"];
#[crate_type = "lib"];

#[feature(macro_rules)];

#[allow(dead_code)];

// TODO: get rid of this once msgpack has got rid of owned vector
#[allow(deprecated_owned_vector)];

#[feature(phase)];
#[phase(syntax, link)] extern crate log;
extern crate msgpack;
extern crate serialize;
extern crate uuid;
extern crate rand;

pub use replica::StateMachine;

mod common;
mod replica;
mod acceptor;
mod leader;

#[cfg(test)]
mod tests {
    extern crate russenger;

    #[test]
    fn do_something() {
        println!("Hello World!");
    }
}