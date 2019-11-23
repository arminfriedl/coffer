#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use std::net::{TcpListener, SocketAddr, TcpStream};
use std::sync::Arc;
use std::io::{Write};
use std::io::Read;
use futures::executor::ThreadPool;

use crate::secrets::{Secrets};

pub struct Channel {
    pub executor: ThreadPool,
    pub address: SocketAddr,
}

impl Channel {
    pub fn listen(self, secrets: Arc<Secrets>) {

        let listener:TcpListener = TcpListener::bind(self.address).unwrap();

        listener.incoming().for_each(|inc| {
            match inc {
                Ok(tcp_stream) => self.executor.spawn_ok(handler(tcp_stream, secrets.clone())),
                Err(e) => error!{"Failed binding incoming connection {}", e}
            }
        })
    }
}

async fn handler(mut stream: TcpStream, secrets: Arc<Secrets>) {
    let mut peek_buf = [0x00; 1];

    while stream.peek(&mut peek_buf).unwrap() != 0 {
        let mut len_buffer = [0x00; std::mem::size_of::<usize>()];
        stream.read_exact(&mut len_buffer);
        let len = usize::from_be_bytes(len_buffer) as u64;
        debug!{"Length {}", len}

        let mut buf: Vec<u8> = Vec::with_capacity(len as usize);
        let mut handle = (&stream).take(len);
        handle.read_to_end(&mut buf);
        debug!{"Read vec {:?}", buf};
        let res: Result<String, serde_cbor::Error> = serde_cbor::from_slice(&buf);

        match res {
            Ok(request) => {

                if let Some(secret) = secrets.get(&request) {
                    writeln!(stream, "{}", secret)
                        .unwrap_or_else(|err| {error!{"Could not write key response to stream {}", err}});
                    debug!{"Wrote {:?}", secret}
                } else {
                    writeln!(stream, "")
                        .unwrap_or_else(|err| {error!{"Could not write key response to stream {}", err}})
                }
            },
            Err(e) => error!{"Could not parse secret request: {}", e}
        };
    }
}
