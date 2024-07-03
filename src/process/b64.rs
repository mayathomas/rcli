use std::{fs::File, io::Read};

use base64::{
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
    Engine,
};

use crate::Base64Format;
pub fn process_encode(input: &str, format: Base64Format) -> anyhow::Result<()> {
    let mut reader = get_reader(input)?;

    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;

    let encode = match format {
        Base64Format::Standard => STANDARD.encode(&buf),
        Base64Format::UrlSafe => URL_SAFE_NO_PAD.encode(&buf),
    };
    println!("{}", encode);
    Ok(())
}

pub fn process_decode(input: &str, format: Base64Format) -> anyhow::Result<()> {
    let mut reader = get_reader(input)?;
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    //buf可能会多出换行
    let buf = buf.trim();

    let _decode = match format {
        Base64Format::Standard => STANDARD.decode(buf)?,
        Base64Format::UrlSafe => URL_SAFE_NO_PAD.decode(buf)?,
    };

    println!("{}", String::from_utf8(_decode)?);
    Ok(())
}

fn get_reader(input: &str) -> anyhow::Result<Box<dyn Read>> {
    //stdin类型和File类型都有一个共同的Read trait，所以可以用Box将返回值封装起来
    let reader: Box<dyn Read> = if input == "-" {
        Box::new(std::io::stdin())
    } else {
        Box::new(File::open(input)?)
    };
    Ok(reader)
}
