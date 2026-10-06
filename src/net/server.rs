//! Mancala over UDP server implementation.

use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};

pub const DEFAULT_PORT: u16 = 25054;
pub const DEFAULT_BIND_ADDR: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
pub const DEFAULT_SOCKET_ADDR: SocketAddr = SocketAddr::new(DEFAULT_BIND_ADDR, DEFAULT_PORT);


pub fn start_server(bind_addr: SocketAddr) {
    let socket = match UdpSocket::bind(bind_addr) {
        Ok(s) => s,
        Err(_) => panic!("Could not bind to {}", bind_addr),
    };

    loop {
        let mut buf = [0; 1024];
        let a = socket.recv_from(&mut buf);
        // TODO: Continue here with tests...
    }
}

pub fn start_server_default() {
    start_server(DEFAULT_SOCKET_ADDR);
}