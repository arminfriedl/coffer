//! # Coffer client
//!
//! Retrieve a secret shard from a `coffer-server`. Secrets in the shard are set
//! as environment variables for the spawned subcommand `cmd`.

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use env_logger;

use std:: {
    net::TcpStream,
    error::Error,
    path::PathBuf,
    io::{Write, Read},
    convert::{TryInto, TryFrom}
};

use coffer_common::{
    coffer::{CofferShard, CofferValue},
    certificate::Certificate
};

use structopt::StructOpt;

/// Client for setting up the environment from coffer server secrets
#[derive(StructOpt, Debug)]
struct Args {
    /// Address of the coffer server
    #[structopt(short, long, env = "COFFER_SERVER_ADDRESS", default_value = "127.0.0.1:9187")]
    server_address: String,

    #[structopt(short, long, parse(from_os_str), env = "COFFER_CLIENT_CERTIFICATE", hide_env_values = true)]
    certificate: PathBuf,

    /// The subcommand spawned by coffer-client
    cmd: String,

    /// Arguments to the subcommand spawned by coffer-client
    cmd_args: Vec<String>
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let args = Args::from_args();

    debug!{"Reading certificate"}
    let cert = Certificate::new_from_cbor(&args.certificate)?;

    debug!{"Connecting to coffer server"}
    let mut stream: TcpStream = TcpStream::connect(args.server_address)?;

    debug!{"Sending hello"}
    let hello = framed(0x00, cert.public_key());
    stream.write_all(&hello)?;

    debug!{"Sending get"}
    let get = framed(0x02, Vec::new());
    stream.write_all(&get)?;

    debug!{"Reading shard"}
    let header = read_header(&mut stream).unwrap();
    let shard = read_message(header.0, &mut stream).unwrap();
    debug!{"Got encrypted shard {:?}", shard}

    debug!{"Sending bye"}
    let bye = framed(0x99, Vec::new());
    stream.write_all(&bye)?;

    debug!{"Decrypting shard"}
    let shard_clear = cert.open(&shard).unwrap();
    let shard_de = serde_cbor::from_slice::<CofferShard>(&shard_clear).unwrap();

    debug!{"Setting environment"}
    for (key, val) in shard_de.0 {
        if let CofferValue::String(val_s) = val {
            std::env::set_var(key.trim(), val_s.trim());
        }
    }

    info!{"Spawning coffer'ed command, reaping coffer"}
    reap_coffer(&args.cmd, &args.cmd_args);

    Err("Could not spawn sub-command".into())
}

/// Replaces the `coffer-client` process image with
/// the subcommand `cmd` with `args`
fn reap_coffer(cmd: &str, args: &[String]) {
    let mut cmd = exec::Command::new(cmd);

    // TODO Push cmd as first arg if not already set?
    cmd.args(args);

    let err = cmd.exec();
    error!{"Could not execute sub-command {}", err};
}

pub fn read_header<T>(reader: &mut T) -> Option<(u64, u8)>
where T: Read
{
    let mut header: [u8; 9] = [0u8;9]; // header buffer
    match reader.read_exact(&mut header)
    {
        Ok(_) => debug!{"Read {} bytes for header", 9},
        Err(err) => {
            error!{"Error while reading header: {}", err}
            return None;
        }
    }

    trace!{"Header buffer {:?}", header}

    let msg_size: u64 = u64::from_be_bytes(
        header[0..8]
            .try_into()
            .unwrap());

    let msg_type: u8 = u8::from_be_bytes(
        header[8..9]
            .try_into()
            .unwrap());

    debug!{"Message size: {}, Message type: {}", msg_size, msg_type}
    Some((msg_size, msg_type))
}

pub fn read_message<T>(msg_size: u64, reader: &mut T) -> Option<Vec<u8>>
where T: Read
{
    // TODO: possible to use unallocated memory instead?
    // -> https://doc.rust-lang.org/beta/std/mem/union.MaybeUninit.html
    // TODO: 32 bit usize? Can't allocate a 64 bit length buffer anyway?
    let mut message = Vec::with_capacity(msg_size.try_into().unwrap());
    // need to set the size, because otherwise it is assumed to be 0, since
    // the vec is allocated but uninitialized at this point, we don't want to
    // pre-allocate a potentially huge buffer with 0x00, so unsafe set size.
    unsafe {message.set_len(msg_size.try_into().unwrap());}

    match reader.read_exact(&mut message)
    {
        Ok(_) => debug!{"Read {} bytes for message", msg_size},
        Err(err) => {
            error!{"Error while reading message: {}", err}
            return None;
        }
    }
    trace!{"Read message {:?}", message}

    Some(message)
}

pub fn framed(msg_type: u8, data: Vec<u8>) -> Vec<u8>
{
    trace!{"Creating frame for type: {:?}, data: {:?}", msg_type, data}

    // TODO magic number
    let mut frame: Vec<u8> = Vec::with_capacity(data.len() + 72);
    unsafe {frame.set_len(8);}

    frame.splice(0..8, u64::try_from(data.len())
                 .unwrap()
                 .to_be_bytes()
                 .iter()
                 .cloned());

    frame.push(msg_type);
    frame.extend(&data);

    frame
}
