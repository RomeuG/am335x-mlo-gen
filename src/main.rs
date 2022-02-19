use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs::File;
use clap::Parser;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    FileCreate(std::io::Error),
    FileOpen(std::io::Error),

    WriteError(std::io::Error),

    FsCopyError,
    FsMetadata,

    BufRead(std::io::Error),

    Seek(u64),
}

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    input: String,

    #[clap(short, long, default_value_t = 0x402F0400)]
    address: i32,

    #[clap(short, long, default_value = "MLO")]
    output: String,
}

fn prng() -> u64 {
    // get time for the seed
    let now = SystemTime::now();
    let timestamp = now.duration_since(UNIX_EPOCH).unwrap();
    let timestamp_ms = timestamp.as_secs();

    let a = 1103515245;
    let c = 12345;
    let m = 2147483647;

    (a * timestamp_ms + c) % m
}

fn main() -> Result<()> {

    let args = Args::parse();

    let binary_name = args.input;
    let mut binary_file = File::open(binary_name).map_err(Error::FileOpen)?;
    let binary_size = binary_file.metadata().map_err(|_| Error::FsMetadata)?.len();
    let binary_size_32 = binary_size as u32;
    let binary_size_32_le = binary_size_32.to_le_bytes();

    let load_address_32_le = args.address.to_le_bytes();

    let tmp_file_name = format!("/tmp/am335x-mlo-gen-{}", prng());
    let mut tmp_file = File::create(&tmp_file_name).map_err(Error::FileCreate)?;

    tmp_file.write_all(&binary_size_32_le).map_err(Error::WriteError)?;
    tmp_file.write_all(&load_address_32_le).map_err(Error::WriteError)?;

    std::io::copy(&mut binary_file, &mut tmp_file).map_err(|_| Error::FsCopyError)?;
    std::fs::copy(&tmp_file_name, args.output).map_err(|_| Error::FsCopyError)?;

    Ok(())
}
