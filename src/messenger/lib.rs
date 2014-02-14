#[crate_id = "messenger"];

#[crate_type = "dylib"];
#[crate_type = "rlib"];

pub trait HasAddress {
    pub fn get_address() -> SocketAddr;
}

pub struct Messenger {
    pub fn new(addr: SocketAddr) -> Messenger {
        
    }
}

impl Messenger {
    pub fn send<T: HasAddress, 'a, U: Encodable<json::Encoder<'a>>>(&mut self, to: T, msg: U) {

    }

    pub fn recv<'a, U: Decodable<json::Decoder>>(&mut self) -> U {

    }
}

pub fn hello() {
    println!("Hello World!");
}