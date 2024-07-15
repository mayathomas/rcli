use clap::Parser;

use rcli::{CmdExecutor, Opts};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let opts = Opts::parse();
    opts.cmd.execute().await?;
    Ok(())
}

#[cfg(test)]
mod test {
    #[test]
    fn ttt() {}
}
