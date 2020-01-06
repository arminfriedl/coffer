//! Public APIs for `coffer-server`

use std::net::ToSocketAddrs;

use tokio::prelude::*;
use tokio::net::TcpListener;

async fn run_server<T: ToSocketAddrs>(sock_addrs: T) {
    let addr = sock_addrs.to_socket_addrs().unwrap().next().unwrap();
    let listener = TcpListener::bind(addr);
}
