#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use env_logger;

use std::convert::TryInto;
use std::path::PathBuf;
use structopt::StructOpt;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::prelude::*;

mod coffer;
mod server;
mod comm;

use comm::Channel;

#[derive(StructOpt, Debug)]
struct Args {
    /// Path to the server certificate. Will be deleted after processing.
    #[structopt(short, long, parse(from_os_str), env = "COFFER_SERVER_CERTIFICATE", hide_env_values = true)]
    certificate: Option<PathBuf>,

    /// Path to an initial secrets file. Will be deleted after processing.
    /// Must be sealed by the public key of the server certificate
    #[structopt(short, long, parse(from_os_str), env = "COFFER_SERVER_SECRETS", hide_env_values = true)]
    secrets: Option<PathBuf>,

    /// Port the coffer server listens on
    #[structopt(short, long, env = "COFFER_SERVER_PORT", default_value = "9187")]
    port: u16,

    /// Address coffer server should bind to
    #[structopt(short, long, parse(try_from_str), env = "COFFER_SERVER_ADDRESS", default_value = "127.0.0.1:9187")]
    ip: SocketAddr,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::from_args();

    _print_banner();

    info!{"Filling coffer"}
    let coffer = coffer::Coffer::new_from_path_encrypted(&args.master, &args.secrets)
        .expect("Could not fill coffer");
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
