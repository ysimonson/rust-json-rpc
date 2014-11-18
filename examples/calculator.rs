extern crate "rust-json-rpc" as json_rpc;
extern crate iron;

use json_rpc::{Server};
use std::io::net::ip::Ipv4Addr;
use iron::{status, Iron, Request, Response, IronResult};

fn main() {
    fn hello_world(_: &mut Request) -> IronResult<Response> {
        Ok(Response::with(status::Ok, "Hello, world"))
    }

    Iron::new(hello_world).listen(Ipv4Addr(127, 0, 0, 1), 3000);
    println!("On 3000");
}
