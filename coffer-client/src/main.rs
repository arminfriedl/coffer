#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use env_logger;
use structopt::StructOpt;
use std::fs::File;
use std::error::Error;
use std::net::TcpStream;
use std::net::SocketAddr;
use std::net::IpAddr;
use std::path::PathBuf;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;

#[derive(StructOpt, Debug)]
struct Args {
    /// The port secsrv is listening on
    #[structopt(short, long, env = "SECSRV_PORT", default_value = "9187")]
    port: u16,

    /// The address secsrv binds to
    #[structopt(short, long, env = "SECSRV_IP", default_value = "127.0.0.1")]
    ip: IpAddr,

    /// Path to the keys file
    #[structopt(parse(from_os_str), env = "SECSRV_SECRETS", hide_env_values = true)]
    secrets: PathBuf,

    /// The subcommand spawned by coffer-client
    cmd: String,

    /// Arguments to the subcommand spawned by coffer-client
    cmd_args: Vec<String>
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let args = Args::from_args();

    info!{"Connecting to coffer server"}
    let addr = SocketAddr::from((args.ip, args.port));
    let stream: TcpStream = TcpStream::connect(addr)?;

    info!{"Parsing key requests"}
    let keys = parse_from_path(&args.secrets)?;

    info!{"Reading secrets"}
    retrieve_secrets(&keys, stream)?;

    info!{"Spawning coffer'ed command, reaping coffer"}
    reap_coffer(&args.cmd, &args.cmd_args);

    Err("Could not spawn sub-command".into())
}

fn retrieve_secrets(keys: &Vec<String>, mut stream: TcpStream) -> Result<(), Box<dyn Error>>{
    for k in keys {
        let buf = serde_cbor::to_vec(&k)?;
        info!{"Sending key request {} as {:?}", k, buf}
        stream.write_all(&buf.len().to_be_bytes())?;
        stream.write_all(&buf)?;

        info!{"Reading response"}
        let mut reader = BufReader::new(&stream); // get buffered reader for line-wise reading from stream

        // read line
        let mut resp = String::new();
        reader.read_line(&mut resp)?;

        info!{"Retrieved secret. Setting environment"}
        std::env::set_var(k.trim(), resp.trim());
    }

    Ok(())
}

fn reap_coffer(cmd: &str, args: &Vec<String>) {
    let mut cmd = exec::Command::new(cmd);

    // TODO Push cmd as first arg if not already set?
    cmd.args(args);

    let err = cmd.exec();
    error!{"Could not execute sub-command {}", err};
}

fn parse_from_path(path: &PathBuf) -> Result<Vec<String>, Box<dyn Error>> {
    let sec_file = File::open(path)?;

    Ok(serde_yaml::from_reader::<_, Vec<String>>(sec_file)?)
}
