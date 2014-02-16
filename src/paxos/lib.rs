#[crate_id = "paxos"];

#[crate_type = "dylib"];
#[crate_type = "rlib"];

#[cfg(test)]
mod tests {
    extern mod russenger;
    
    #[test]
    fn do_something() {
        println!("Hello World!");
    }
}