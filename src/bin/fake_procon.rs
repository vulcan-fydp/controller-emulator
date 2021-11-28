use anyhow::Result;
use nix::sys::stat;
use nix::unistd::mkfifo;
use std::env;
use std::fs::{remove_file, OpenOptions};
use std::io::prelude::*;
use std::io::stdin;
use std::path::Path;
use std::process::exit;
use std::thread;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    assert!(args.len() == 2);

    let file_in = Path::new(&args[1]).with_extension("in");
    let file_out = Path::new(&args[1]).with_extension("out");

    let fi = file_in.clone();
    let fo = file_out.clone();
    ctrlc::set_handler(move || {
        let _ = remove_file(&fi);
        let _ = remove_file(&fo);

        exit(0);
    })?;

    let _ = remove_file(&file_in);
    let _ = remove_file(&file_out);

    mkfifo(&file_in, stat::Mode::from_bits_truncate(0o644))?;
    mkfifo(&file_out, stat::Mode::from_bits_truncate(0o644))?;

    let mut f_in = OpenOptions::new().read(true).write(true).open(&file_in)?;
    let mut f_out = OpenOptions::new().read(true).write(true).open(&file_out)?;

    thread::spawn(move || loop {
        let mut buf = [0; 64];
        let read = f_in.read(&mut buf).unwrap();
        if read > 0 {
            println!("ns recv: {:?}", &buf[..read]);
        }
    });

    loop {
        let mut _in = String::new();
        let read = stdin().read_line(&mut _in)?;
        _in.pop();

        if read == 1 {
            break;
        }

        let mut in_bytes = vec![0u8; _in.len() / 2];
        match hex::decode_to_slice(&_in, &mut in_bytes) {
            Ok(()) => {
                let _ = f_out.write(&in_bytes);
            }
            Err(e) => {
                println!("Invalid input: \"{}\" {:?}", &_in, e);
            }
        }
    }

    let _ = remove_file(&file_in);
    let _ = remove_file(&file_out);

    Ok(())
}
