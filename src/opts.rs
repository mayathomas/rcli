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
}

#[derive(Debug, Parser)]
pub struct CsvOpts {
    #[arg(short, long, value_parser = verify_input_file)]
    pub input: String,

    //default_value会自动为实现了From trait的做into转换, "output.json"是&str类型，而我们需要String类型，所以要default_value。"output.json".into()
    #[arg(short, long, default_value = "output.json")]
    pub output: String,

    #[arg(short, long, default_value = ",")]
    pub delimiter: char,

    #[arg(short = 'r', long, default_value_t = true)]
    pub header: bool,
}

//String字面量是生命周期为static的&str
fn verify_input_file(filename: &str) -> Result<String, &'static str> {
    if std::path::Path::new(filename).exists() {
        Ok(filename.into())
    } else {
        Err("File does not exist")
    }
}
