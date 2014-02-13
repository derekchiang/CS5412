#[crate_id = "messenger"];

#[crate_type = "dylib"];
#[crate_type = "rlib"];

pub fn hello() {
    println!("Hello World!");
}