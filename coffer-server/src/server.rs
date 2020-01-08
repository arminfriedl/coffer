#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use quick_error::quick_error;

use tokio::net::{TcpListener, TcpStream};
use tokio::stream::StreamExt;
use tokio::sync::RwLock;

use std::net::{ToSocketAddrs, SocketAddr};
use std::sync::Arc;

use coffer_common::keyring::Keyring;
use coffer_common::coffer::Coffer;
use coffer_common::certificate::{Certificate, CertificateError};

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
    keyring: Arc<RwLock<Keyring>>,
    coffer: Arc<RwLock<C>>
}

impl <C> Server<C>
where C: Coffer + Send + Sync + 'static
{
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
                    Ok(mut tcp_stream) => {
                        debug!{"Connection ok"}
                        debug!{"Spawning off connection handler"}

                        let keyring = self.keyring.clone();
                        let coffer = self.coffer.clone();
                        tokio::spawn(Self::handle_connection(keyring, coffer, tcp_stream));
                    }
                    Err(err) => error!{"Connection could not be established {}", err}
                }
                debug!{"Waiting for new connections"}
            }
        };

        server.await
    }

    async fn handle_connection(keyring: Arc<RwLock<Keyring>>,
                               coffer: Arc<RwLock<C>>,
                               mut tcp_stream: TcpStream)
    {
        let (reader, mut writer) = tcp_stream.split();
    }
}

pub struct ServerBuilder<C>
where C: Coffer
{
    keyring: Option<Keyring>,
    coffer: Option<C>
}

impl <'a, C> ServerBuilder<C>
where C: Coffer + Default
{
    pub fn new() -> ServerBuilder<C> {
        ServerBuilder {
            keyring: None,
            coffer: None
        }
    }

    pub fn with_keyring(mut self, keyring: Option<Keyring>) -> ServerBuilder<C> {
        self.keyring = keyring;
        self
    }

    pub fn with_coffer(mut self, coffer: Option<C>) -> ServerBuilder<C> {
        self.coffer = coffer;
        self
    }

    pub fn build(self) -> Result<Server<C>, ServerError> {
        let keyring = match self.keyring {
            Some(k) => Arc::new(RwLock::new(k)),
            None => {let cert = Certificate::new()?;
                     Arc::new(RwLock::new(Keyring::new(cert)))}
        };

        let coffer = Arc::new(RwLock::new(self.coffer.unwrap_or_else(|| { C::default() } )));

        Ok(Server {keyring, coffer})
    }
}
