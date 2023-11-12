use clap::Parser;
use std::net::IpAddr;
use std::path::PathBuf;

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
    for i in 1..=args.ips.len() {
        let mut path = args.config_base_path.clone();
        path.push(format!("{}{}", args.hostname_prefix, i + 1));
        path.push("etc/hosts");

        let mut hosts = EtcHosts::from_str(&std::fs::read_to_string(path)?)?;
        for (j, &ip) in args.ips.iter().enumerate() {
            hosts.add_data(ip, &format!("{}{}", args.hostname_prefix, j + 1));
        }
        print!("{}", hosts.to_string());
    }
    Ok(())
}
