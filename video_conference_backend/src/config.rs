use std::net::{SocketAddr, IpAddr, Ipv4Addr};

pub fn get_signaling_server_addr() -> SocketAddr {
    SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 
        3030
    )
}