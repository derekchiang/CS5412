#[crate_id = "paxos"];

#[crate_type = "dylib"];
#[crate_type = "rlib"];

#[cfg(test)]
mod tests {
    extern mod messenger;
    
    #[test]
    fn do_something() {
        messenger::hello();
    }
}