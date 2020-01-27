#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use env_logger;

use std::path::PathBuf;
use std::fs::File;
use std::io::{Read};
use structopt::StructOpt;
use std::net::SocketAddr;

use coffer_common::keyring::Keyring;
use coffer_common::coffer::Coffer;

mod server;
mod coffer_map;
mod protocol;

use server::Server;
use coffer_map::CofferMap;

#[derive(StructOpt, Debug)]
struct Args {
    /// Path to the server certificate. Will be deleted after processing.
    #[structopt(short, long, parse(from_os_str), env = "COFFER_SERVER_CERTIFICATE", hide_env_values = true)]
    certificate: PathBuf,

    /// Path to secrets file. Will be deleted after processing.
    /// Must be sealed by the public key of the server certificate
    #[structopt(short, long, parse(from_os_str), env = "COFFER_SERVER_SECRETS", hide_env_values = true)]
    secrets: PathBuf,

    /// Address, the coffer server should bind to
    #[structopt(short, long, parse(try_from_str), env = "COFFER_SERVER_ADDRESS", default_value = "127.0.0.1:9187")]
    address: SocketAddr,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::from_args();

    _print_banner();

    // create keyring from server certificate
    let mut keyring = Keyring::new_from_path(&args.certificate);

    // decrypt secrets file and put into coffer
    let mut secrets_file = File::open(&args.secrets).unwrap();
    let mut secrets_buf = Vec::new();
    secrets_file.read_to_end(&mut secrets_buf).unwrap();
    let secrets_buf_clear = String::from_utf8(keyring.open(&secrets_buf).unwrap()).unwrap();

    // read known client ids from secrets file
    keyring.add_known_keys_toml(&secrets_buf_clear).unwrap();

    // read secrets from secrets file
    let coffer = CofferMap::from_toml(&secrets_buf_clear);

    // start server
    let server = Server::new(keyring, coffer);
    server.run(args.address).await;
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
