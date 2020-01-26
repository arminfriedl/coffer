use std::path::PathBuf;
use structopt::StructOpt;

mod certificate;
mod encrypt;
mod client;

#[derive(StructOpt, Debug)]
enum Args {
    Certificate {
        #[structopt(short, long, parse(from_os_str))]
        out: PathBuf,
        #[structopt(short, long)]
        info: bool,
    },
    Encrypt {
        #[structopt(short, long, parse(from_os_str))]
        certificate: PathBuf,
        #[structopt(short, long, parse(from_os_str))]
        yaml: PathBuf,
        #[structopt(short, long, parse(from_os_str))]
        out: PathBuf
    },
    Client {
        #[structopt(short, long, parse(from_os_str))]
        certificate: PathBuf,
    }
}

fn main() {
    let args: Args = Args::from_args();

    match args {
        Args::Certificate {out, info} => {
            if info {  certificate::info(out) }
            else { certificate::generate_key(out) }
        }
        Args::Encrypt {certificate, yaml, out} => {
            encrypt::encrypt_yaml(yaml, out, certificate)
        }
        Args::Client {certificate} => {
            client::print_get(certificate)
        }
    }
}
