use clap::Parser;
use std::{
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
};

mod hosts;

use crate::hosts::EtcHosts;

#[derive(Debug, Parser)]
struct Args {
    ips: Vec<IpAddr>,
    #[arg(long, default_value = "config")]
    config_base_path: PathBuf,
    #[arg(long, default_value = "is")]
    hostname_prefix: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    for i in 0..args.ips.len() {
        let mut path = args.config_base_path.clone();
        path.push(format!("{}{}", args.hostname_prefix, i + 1));
        path.push("etc/hosts");

        let mut hosts = EtcHosts::from_str(&std::fs::read_to_string(&path)?)?;
        for (j, &ip) in args.ips.iter().enumerate() {
            // loopback addr
            let ip = if i == j {
                IpAddr::from(Ipv4Addr::new(127, 0, 0, 1))
            } else {
                ip
            };
            hosts.add_data(ip, &format!("{}{}", args.hostname_prefix, j + 1));
        }
        std::fs::write(&path, hosts.to_string())?;
    }
    Ok(())
}
