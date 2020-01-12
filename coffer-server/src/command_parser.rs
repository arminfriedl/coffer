#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use tokio::io::AsyncRead;

use quick_error::quick_error;

quick_error! {
    #[derive(Debug)]
    pub enum CommandParserError {
        Msg(err: &'static str) {
            from(err)
                display("{}", err)
        }
        Other(err: Box<dyn std::error::Error>) {
            cause(&**err)
        }
    }
}

enum Command {
    None
}

struct CommandParser<T>
where T: AsyncRead {
    reader: T
}

impl Stream for CommandParser {
    type Item = Command;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Option<Self::Item>> {
        Poll::Ready(Some(Command::None))
    }
}
