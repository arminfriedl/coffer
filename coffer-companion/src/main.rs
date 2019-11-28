use std::error::Error;
use std::fs::File;
use std::io::Write;
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

fn main() -> Result<(), Box<dyn Error>> {
    let args: Args = Args::from_args();

    match args {
        Args::Generate {out} => generate::generate_key(out),
        Args::Encrypt {yaml, out, masterkey} => encrypt::generate_encrypted_secrets(yaml, out, masterkey)
    }

    let secreta = "ABC".to_owned();
    let mut f = File::create("./keyreq_a.cbor")?;
    let buf = serde_cbor::to_vec(&secreta)?;
    f.write(&buf.len().to_be_bytes())?;
    f.write(&buf)?;

    let secretb = "XYZ".to_owned();
    let mut f = File::create("./keyreq_b.cbor")?;
    let buf = serde_cbor::to_vec(&secretb)?;
    f.write(&buf.len().to_be_bytes())?;
    f.write(&buf)?;

    let secs = vec!["ABC", "XYZ"];
    let f = File::create("./secreq.yaml")?;
    serde_yaml::to_writer(f, &secs)?;

    Ok(())
}
