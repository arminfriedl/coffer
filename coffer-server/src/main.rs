#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use env_logger;

use std::convert::TryInto;
use futures::executor::ThreadPool;
use std::path::PathBuf;
use structopt::StructOpt;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::sync::Arc;

mod coffer;
mod comm;

use comm::Channel;

#[derive(StructOpt, Debug)]
struct Args {
    /// Path to the master key file. Will be deleted after processing.
    #[structopt(short, long, parse(from_os_str), env = "SECSRV_MASTER", hide_env_values = true)]
    master: PathBuf,

    /// Path to the secret keys file. Will be deleted after processing.
    /// Must be encrypted with the public key of the master key
    #[structopt(short, long, parse(from_os_str), env = "SECSRV_KEYS", hide_env_values = true)]
    secrets: PathBuf,

    /// The port secsrv listens on
    #[structopt(short, long, env = "SECSRV_PORT", default_value = "9187")]
    port: u16,

    /// The address secsrv binds to
    #[structopt(short, long, env = "SECSRV_IP", default_value = "127.0.0.1")]
    ip: IpAddr,

    /// Prevent deletion of key files
    #[structopt(long)]
    keep_keys: bool
}

fn main() {
    env_logger::init();
    let args = Args::from_args();

    _print_banner();

    info!{"Setting up executor"}
    let address: SocketAddr = (args.ip, args.port).try_into()
        .expect("Parsing binding address failed");
    let executor = ThreadPool::new()
        .expect("Setting up executor failed");

    info!{"Filling coffer"}
    let coffer = coffer::Coffer::new_from_path_encrypted(&args.master, &args.secrets, args.keep_keys)
        .expect("Could not fill coffer");

    debug!{"Connecting on {}", address}
    let channel = Channel {executor, address, coffer: Arc::from(coffer)};
    channel.listen();
}

fn _print_banner() {
    println!{r#"


 @@@@@@@   @@@@@@   @@@@@@@@  @@@@@@@@  @@@@@@@@  @@@@@@@
@@@@@@@@  @@@@@@@@  @@@@@@@@  @@@@@@@@  @@@@@@@@  @@@@@@@@
!@@       @@!  @@@  @@!       @@!       @@!       @@!  @@@
!@!       !@!  @!@  !@!       !@!       !@!       !@!  @!@
!@!       @!@  !@!  @!!!:!    @!!!:!    @!!!:!    @!@!!@!
!!!       !@!  !!!  !!!!!:    !!!!!:    !!!!!:    !!@!@!
:!!       !!:  !!!  !!:       !!:       !!:       !!: :!!
:!:       :!:  !:!  :!:       :!:       :!:       :!:  !:!
 ::: :::  ::::: ::   ::        ::        :: ::::  ::   :::
 :: :: :   : :  :    :         :        : :: ::    :   : :


"#}

}
