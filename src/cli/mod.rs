mod base64;
mod csv;
mod genpass;

pub use self::{
    base64::{Base64Format, Base64SubCommand},
    csv::{CsvOpts, OutputFormat},
    genpass::GenPassOpts,
};
use clap::Parser;

#[derive(Debug, Parser)]
#[command(name="rcli", version, author, about, long_about=None)]
pub struct Opts {
    //子命令
    #[command(subcommand)]
    pub cmd: SubCommand,
}

#[derive(Debug, Parser)]
pub enum SubCommand {
    //子命令，-- csv，中间有个空格
    #[command(name = "csv", about = "Show CSV, or convert CSV to other formats")]
    Csv(CsvOpts),
    #[command(name = "genpass", about = "Generate a random password")]
    GenPass(GenPassOpts),
    #[command(subcommand)]
    Base64(Base64SubCommand),
}

//String字面量是生命周期为static的&str
fn verify_input_file(filename: &str) -> Result<String, &'static str> {
    if filename == "-" || std::path::Path::new(filename).exists() {
        Ok(filename.into())
    } else {
        Err("File does not exist")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_input_file() {
        assert_eq!(verify_input_file("-"), Ok("-".into()));
        assert_eq!(verify_input_file("Cargo.toml"), Ok("Cargo.toml".into()));
        assert_eq!(
            verify_input_file("nonexistent.txt"),
            Err("File does not exist")
        );
    }
}
