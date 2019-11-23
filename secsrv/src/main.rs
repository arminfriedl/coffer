#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use env_logger;

use std::convert::TryInto;
use std::error::Error;
use futures::executor::ThreadPool;
use std::path::PathBuf;
use structopt::StructOpt;
use std::net::IpAddr;
use std::net::SocketAddr;

mod comm;
mod keyring;
mod secrets;

use comm::Channel;

#[derive(StructOpt, Debug)]
struct Args {
    /// Path to the master key file. Will be deleted after processing.
    #[structopt(short, long, parse(from_os_str), env = "SECSRV_MASTER", hide_env_values = true)]
    master: PathBuf,

    /// Path to the secret keys file. Will be deleted after processing.
    #[structopt(long, parse(from_os_str), env = "SECSRV_KEYS", hide_env_values = true)]
    keys: PathBuf,

    /// The port secsrv is listening on
    #[structopt(short, long, env = "SECSRV_PORT", default_value = "9187")]
    port: u16,

    /// The address secsrv binds to
    #[structopt(short, long, env = "SECSRV_IP", default_value = "127.0.0.1")]
    ip: IpAddr,

    /// Prevent deletion of key files
    #[structopt(short)]
    keep_keys: bool
}

fn main() -> Result<(), Box<dyn Error>>{
    env_logger::init();
    let args = Args::from_args();

    _print_banner();

    debug!{"Parsing master key from {}", args.master.display()}
    let _master_key = keyring::parse_from_path(&args.master, args.keep_keys)?;

    debug!{"Parsing secrets from {}", args.keys.display()}
    let secrets = secrets::parse_from_path(&args.keys, args.keep_keys)?;

    info!{"Setting up the connection pool"}
    let executor = ThreadPool::new()?;
    let address: SocketAddr = (args.ip, args.port).try_into()?;
    debug!{"Connecting on {}", address}
    let channel = Channel {executor, address};
    channel.listen(secrets.into());

    Ok(())
}

fn _print_banner() {
    info!{r#"


 ___  ___  ___ ___  ___ _ ____   __
/ __|/ _ \/ __/ __|/ _ \ '__\ \ / /
\__ \  __/ (__\__ \  __/ |   \ V /
|___/\___|\___|___/\___|_|    \_/


"#}

}
