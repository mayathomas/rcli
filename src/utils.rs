use std::{fs::File, io::Read};

pub fn get_reader(input: &str) -> anyhow::Result<Box<dyn Read>> {
    //stdin类型和File类型都有一个共同的Read trait，所以可以用Box将返回值封装起来
    let reader: Box<dyn Read> = if input == "-" {
        Box::new(std::io::stdin())
    } else {
        Box::new(File::open(input)?)
    };
    Ok(reader)
}

pub fn get_content(input: &str) -> anyhow::Result<Vec<u8>> {
    let mut reader = get_reader(input)?;
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    Ok(buf)
}
