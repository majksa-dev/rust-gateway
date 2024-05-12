use std::{env, net::TcpListener};

pub fn random_listeners(amount: u16) -> Vec<TcpListener> {
    (0..amount)
        .map(|_| TcpListener::bind("127.0.0.1:0").unwrap())
        .collect()
}

pub fn get_random_ports(amount: u16) -> Vec<u16> {
    random_listeners(amount)
        .into_iter()
        .map(|listener| {
            let port = listener.local_addr().unwrap().port();
            drop(listener);
            port
        })
        .collect()
}

pub fn setup_env() {
    env::set_var("RUST_BACKTRACE", "0");
    essentials::install();
}
