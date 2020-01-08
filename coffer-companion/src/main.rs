use std::path::PathBuf;
use structopt::StructOpt;

mod generate;
mod encrypt;

#[derive(StructOpt, Debug)]
enum Args {
    Generate {
        #[structopt(short, long, parse(from_os_str))]
        out: PathBuf
    },
    Encrypt {
        #[structopt(short, long, parse(from_os_str))]
        yaml: PathBuf,
        #[structopt(short, long, parse(from_os_str))]
        out: PathBuf,
        #[structopt(short, long, parse(from_os_str))]
        masterkey: PathBuf,
    }
}

fn main() {
    let args: Args = Args::from_args();

    match args {
        Args::Generate {out} => generate::generate_key(out),
        Args::Encrypt {yaml, out, masterkey} => {}
    }
}
