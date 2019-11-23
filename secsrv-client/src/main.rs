#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use env_logger;
use structopt::StructOpt;
use std::fs::File;
use std::error::Error;
use std::net::TcpStream;
use std::net::SocketAddr;
use std::io::Read;
use std::net::IpAddr;
use std::path::PathBuf;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;

#[derive(StructOpt, Debug)]
struct Args {
    /// Path to the keys file
    #[structopt(short, long, parse(from_os_str), env = "SECSRV_SECRETS", hide_env_values = true)]
    secrets: PathBuf,

    /// The port secsrv is listening on
    #[structopt(short, long, env = "SECSRV_PORT", default_value = "9187")]
    port: u16,

    /// The address secsrv binds to
    #[structopt(short, long, env = "SECSRV_IP", default_value = "127.0.0.1")]
    ip: IpAddr
}

type Secrets = Vec<String>;

fn main() -> Result<(), Box<dyn Error>>{
    env_logger::init();
    let args = Args::from_args();

    info!{"Parsing sec requests"}

    let secrets = parse_from_path(&args.secrets)?;

    info!{"Connecting"}
    let mut channel: TcpStream = TcpStream::connect(SocketAddr::from((args.ip, args.port)))?;

    info!{"Reading secrets"}

    for s in secrets {
        let buf = serde_cbor::to_vec(&s)?;
        channel.write_all(&buf.len().to_be_bytes())?;
        channel.write_all(&buf)?;
        info!{"Wrote secret {} as {:?}", s, buf}

        let mut resp = String::new();
        let mut reader = BufReader::new(&channel);
        reader.read_line(&mut resp);
        println!{"Resp: {:?}", resp};
    }

    Ok(())
}

pub fn parse_from_path(path: &PathBuf) -> Result<Secrets, Box<dyn Error>> {
    let sec_file = File::open(path)?;

    Ok(serde_yaml::from_reader::<_,Secrets>(sec_file)?)
}
