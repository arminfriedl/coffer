#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use std::path::PathBuf;
use std::convert::{TryFrom, TryInto};
use std::net::{TcpStream};
use std::io::{Write, Read};

use coffer_common::certificate::Certificate;
use coffer_common::coffer::CofferShard;

use serde_cbor;

pub fn print_get(out: PathBuf) {
    let cert = Certificate::new_from_cbor(out).unwrap();

    let hello = framed(0x00, cert.public_key());
    let get = framed(0x02, Vec::new());
    let bye = framed(0x99, Vec::new());

    let mut listener = TcpStream::connect("127.0.0.1:9187").unwrap();
    listener.write_all(&hello).unwrap();

    listener.write_all(&get).unwrap();

    let header = read_header(&mut listener).unwrap();
    let shard = read_message(header.0, &mut listener).unwrap();
    debug!{"Got encrypted shard {:?}", shard}

    listener.write_all(&bye).unwrap();

    let shard_clear = cert.open(&shard).unwrap();
    let shard_de = serde_cbor::from_slice::<CofferShard>(&shard_clear).unwrap();

    println!{"{:?}", shard_de}
}

fn framed(msg_type: u8, data: Vec<u8>) -> Vec<u8>
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

fn read_header<T>(reader: &mut T) -> Option<(u64, u8)>
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

fn read_message<T>(msg_size: u64, reader: &mut T) -> Option<Vec<u8>>
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
