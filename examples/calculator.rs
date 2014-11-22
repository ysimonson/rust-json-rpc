extern crate "rust-json-rpc" as json_rpc;
extern crate iron;

use json_rpc::{Server};
use std::io::net::ip::Ipv4Addr;
use iron::{status, Iron, Request, Response, IronResult};

fn main() {
    let s = Server::new();
    Iron::new(s).listen(Ipv4Addr(127, 0, 0, 1), 3000);
    println!("On 3000");
}
