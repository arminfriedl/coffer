use std::path::PathBuf;
use structopt::StructOpt;

mod certificate;
mod encrypt;

#[derive(StructOpt, Debug)]
enum Args {
    Certificate {
        #[structopt(parse(from_os_str))]
        path: PathBuf,
    },
    Encrypt {
        #[structopt(short, long, parse(from_os_str))]
        certificate: PathBuf,
        #[structopt(short, long, parse(from_os_str))]
        yaml: PathBuf,
        #[structopt(short, long, parse(from_os_str))]
        out: PathBuf
    },
    Info {
        #[structopt(parse(from_os_str))]
        path: PathBuf
    }
}

fn main() {
    let args: Args = Args::from_args();

    match args {
        Args::Certificate {path} => {
            certificate::generate_key(path)
        }
        Args::Encrypt {certificate, yaml, out} => {
            encrypt::encrypt_yaml(yaml, out, certificate)
        }
        Args::Info {path} => {
            certificate::info(path)
        }
    }
}
