#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use env_logger;

use std::path::PathBuf;
use structopt::StructOpt;
use std::net::SocketAddr;

use coffer_common::certificate::Certificate;
use coffer_common::keyring::Keyring;

mod server;
mod coffer_map;
mod protocol;

use server::ServerBuilder;
use coffer_map::CofferMap;

#[derive(StructOpt, Debug)]
struct Args {
    /// Path to the server certificate. Will be deleted after processing.
    #[structopt(short, long, parse(from_os_str), env = "COFFER_SERVER_CERTIFICATE", hide_env_values = true)]
    certificate: Option<PathBuf>,

    /// Path to secrets file. Will be deleted after processing.
    /// Must be sealed by the public key of the server certificate
    #[structopt(short, long, parse(from_os_str), env = "COFFER_SERVER_SECRETS", hide_env_values = true)]
    secrets: Option<PathBuf>,

    /// Address, the coffer server should bind to
    #[structopt(short, long, parse(try_from_str), env = "COFFER_SERVER_ADDRESS", default_value = "127.0.0.1:9187")]
    address: SocketAddr,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::from_args();

    _print_banner();

    let server = ServerBuilder::new()
        .with_keyring(args.certificate.and_then(|cert_path| Some(Keyring::new(Certificate::from(cert_path)))))
        .with_coffer(Some(CofferMap::new()))
        .build()
        .expect("Couldn't build server");

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
