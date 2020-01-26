#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use quick_error::quick_error;

use tokio::net::{TcpListener};
use tokio::stream::StreamExt;

use std::net::{ToSocketAddrs, SocketAddr};
use std::sync::Arc;

use coffer_common::keyring::Keyring;
use coffer_common::coffer::Coffer;
use coffer_common::certificate::CertificateError;

use crate::protocol::Protocol;

quick_error! {
    #[derive(Debug)]
    pub enum ServerError {
        Cert(err: CertificateError) {
            from()
        }
        Msg(err: &'static str) {
            from(err)
            display("{}", err)
        }
        Other(err: Box<dyn std::error::Error>) {
            cause(&**err)
        }
    }
}

pub struct Server<C>
where C: Coffer
{
    keyring: Arc<Keyring>,
    coffer: Arc<C>
}

impl <C> Server <C>
where C: Coffer + Send + Sync + 'static
{

    pub fn new(keyring: Keyring, coffer: C) -> Self {
        Server { keyring: Arc::new(keyring),
                 coffer: Arc::new(coffer) }
    }

    pub async fn run<T>(self, addr: T)
    where T: ToSocketAddrs
    {
        debug!{"Building socket"}
        let socket: SocketAddr = addr
            .to_socket_addrs()
              .expect("Could not convert to socket")
            .next()
              .expect("No socket could be built");

        debug!{"Binding to socket {:?}", socket}
        let mut listener = TcpListener::bind(socket).await
            .expect(format!{"Could not bind to socket {}", socket}.as_str());

        let server = async move {
            let mut incoming = listener.incoming();

            debug!{"Starting connection loop"}
            while let Some(connection) = incoming.next().await {
                debug!{"New incoming connection"}
                match connection {
                    Ok(tcp_stream) => {
                        debug!{"Connection ok\nSpawning off connection handler"}

                        let keyring = self.keyring.clone();
                        let coffer = self.coffer.clone();

                        let protocol = Protocol::new(tcp_stream, coffer, keyring);

                        tokio::spawn(async move {
                            protocol.run().await;
                        });

                    }
                    Err(err) => error!{"Connection could not be established {}", err}
                }
                debug!{"Waiting for new connections"}
            }
        };

        server.await
    }
}
