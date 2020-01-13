#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use std::sync::Arc;
use std::convert::TryInto;
use std::convert::TryFrom;
use std::net::Shutdown;

use tokio::prelude::*;
use tokio::net::TcpStream;
use tokio::sync::RwLock;

use serde_cbor;

use quick_error::quick_error;

use coffer_common::coffer::Coffer;
use coffer_common::coffer::CofferValue;
use coffer_common::coffer::CofferPath;
use coffer_common::keyring::Keyring;

quick_error! {
    #[derive(Debug)]
    pub enum ProtocolError {
        Msg(err: &'static str) {
            from(err)
                display("{}", err)
        }
        Other(err: Box<dyn std::error::Error>) {
            cause(&**err)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum State {
    Start,
    Link,
    Error,
    End
}

#[derive(Debug)]
enum Request {
    Hello(Vec<u8>),
    Put(Vec<u8>),
    Get(Vec<u8>),
    Bye,
    Error
}

pub struct Protocol<C>
where C: Coffer
{
    stream: TcpStream,
    coffer: Arc<RwLock<C>>,
    keyring: Arc<RwLock<Keyring>>,
    client: Option<Vec<u8>>,
    state: State
}

impl<C> Protocol<C>
where C: Coffer
{
    pub fn new(
        stream: TcpStream,
        coffer: Arc<RwLock<C>>,
        keyring: Arc<RwLock<Keyring>>
    ) -> Protocol<C>
    {

        let state = State::Start;
        let client = None;
        Protocol {stream, coffer, keyring, client, state}
    }

    pub async fn run(mut self) {
        while self.state != State::End
        {
            debug!{"In state: {:?}", self.state}
            let event = self.event().await;
            self.transit(event).await;
        }

        self.stream.shutdown(Shutdown::Both).unwrap();
    }

    async fn event(&mut self) -> Request {
        let (mut reader, _writer) = self.stream.split();

        // TODO restrict msg_size more, otherwise bad client could bring server
        //      to allocate vast amounts of memory
        let (msg_size, msg_type) = Self::read_header(&mut reader).await
            .unwrap();

        // TODO only read message if message expected by message type
        //      currently relies on client sending good message
        //      (0x00 message size)
        let message = Self::read_message(msg_size, &mut reader).await
            .unwrap();

        match msg_type {
            0x00 => Request::Hello(message),
            0x02 => Request::Put(message),
            0x03 => Request::Get(message),
            0x63 => Request::Bye,
            0xff => Request::Error,
               _ => Request::Error
        }
    }

    async fn read_header<T>(reader: &mut T) -> Option<(u64, u8)>
    where T: AsyncRead + Unpin
    {
        let mut header: [u8; 9] = [0u8;9]; // header buffer
        match reader.read_exact(&mut header).await
        {
            Ok(size) => debug!{"Read {} bytes for header", size},
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

    async fn read_message<T>(msg_size: u64, reader: &mut T) -> Option<Vec<u8>>
    where T: AsyncRead + Unpin
    {
        // TODO: possible to use unallocated memory instead?
        // -> https://doc.rust-lang.org/beta/std/mem/union.MaybeUninit.html
        // TODO: 32 bit usize? Can't allocate a 64 bit length buffer anyway?
        let mut message = Vec::with_capacity(msg_size.try_into().unwrap());
        // need to set the size, because otherwise it is assumed to be 0, since
        // the vec is allocated but uninitialized at this point, we don't want to
        // pre-allocate a potentially huge buffer with 0x00, so unsafe set size.
        unsafe {message.set_len(msg_size.try_into().unwrap());}

        match reader.read_exact(&mut message).await
        {
            Ok(size) => debug!{"Read {} bytes for message", size},
            Err(err) => {
                error!{"Error while reading message: {}", err}
                return None;
            }
        }
        trace!{"Read message {:?}", message}

        Some(message)
    }

    async fn transit(&mut self, event: Request) {
        match (&self.state, event) {
            (State::Start, Request::Hello(pk)) => {
                debug!{"Reading public key"}
                self.keyring.write().await
                    .add_known_key(&pk)
                    .unwrap();
                self.client = Some(pk);
                self.state = State::Link;
            }

            (State::Link, Request::Get(req)) => {
                debug!{"Writing response"}
                let req = serde_cbor::from_slice(
                    &self.keyring.read().await
                        .open(&req)
                        .unwrap()
                ).unwrap();

                let res = self.coffer.read().await
                    .get(req)
                    .unwrap();

                let response = self.keyring.read().await
                    .seal(
                        &self.client.as_ref().unwrap(),
                        &serde_cbor::to_vec(&res).unwrap()
                    ).unwrap();

                // TODO magic number
                let frame = Self::framed(0x05u8, response).await;
                trace!{"OkGet Frame: {:?}", frame}
                // TODO Proper result handling
                self.stream.write_all(&frame).await.unwrap();

                self.state = State::Link;
            }

            (State::Link, Request::Put(put)) => {
                debug!{"Putting secrets"}
                let put: Vec<(CofferPath, CofferValue)> =
                    serde_cbor::from_slice(
                        &self.keyring.read().await
                            .open(&put)
                            .unwrap()
                    ).unwrap();

                for (coffer_path, coffer_value) in put {
                    self.coffer.write().await
                        .put(coffer_path, coffer_value)
                        .unwrap();
                }

                self.state = State::Link;
            }

            (_, Request::Bye) => self.state = State::End,

            (_, Request::Error) => self.state = State::End,

            _ => self.state = State::End
        }
    }

    async fn framed(msg_type: u8, data: Vec<u8>) -> Vec<u8> {
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
}
