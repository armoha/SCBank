use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use failure::Error;
use flate2::{Compression, write::ZlibEncoder};

#[cfg(windows)]
fn add_icon() -> Result<(), Error> {
    use winres;
    let mut res = winres::WindowsResource::new();
    res.set_icon("resources/SCBank.ico");
    res.compile()?;
    Ok(())
}

#[cfg(windows)]
fn main() -> Result<(), Error> {
    add_icon()?;
    compress_assets()?;
    Ok(())
}

#[cfg(not(windows))]
fn main() -> Result<(), Error> {
    compress_assets()?;
    Ok(())
}

fn compress_assets() -> Result<(), Error> {
    let inputs = [
        "ko-KR.ftl",
        "en-US.ftl",
        "zh-CN.ftl",
        "raw/bc2017console.png",
        "raw/bl.ttf",
        "raw/folder.png",
        "raw/homepage.jpg",
        "raw/hover.png",
        "raw/language.png",
        "raw/mousedown2.wav",
        "raw/update.png",
        "raw/defaultLogo.jpg",
    ];
    let subfolder = "resources/";
    let cmp = "cmp/";
    for s in &inputs {
        let output_filename: &str = if &s[0..4] == "raw/" {
            &s[4..]
        } else { &s };
        let output_path = &format!("{}{}{}", subfolder, cmp, output_filename);
        let output_path = Path::new(output_path);
        /*match File::open(&output_path) {
            Ok(_) => {
                println!("Skip asset compression");
                continue;
            },
            Err(_) => (),
        }*/

        let mut e = ZlibEncoder::new(Vec::new(), Compression::best());
        let input_path = &format!("{}{}", subfolder, s);
        let input_path = Path::new(input_path);
        let mut file = match File::open(&input_path) {
            Ok(f) => f,
            Err(_) => continue,
        };
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        match e.write_all(&mut buffer) {
            Ok(_) => (),
            Err(_) => continue,
        };
        let mut file = File::create(&output_path)?;
        file.write_all(&e.finish()?)?;
    }
    Ok(())
}
