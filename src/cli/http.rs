use std::path::PathBuf;

use clap::Parser;

use crate::CmdExecutor;

use super::verify_path;

#[derive(Debug, Parser)]
pub enum HttpSubCommand {
    //不写name会默认使用首字母小写的命令名
    #[command(about = "Serve a directory over HTTP")]
    Serve(HttpServeOpts),
}

#[derive(Debug, Parser)]
pub struct HttpServeOpts {
    #[arg(long, value_parser = verify_path, default_value = ".")]
    pub dir: PathBuf,
    #[arg(long, default_value_t = 8080)]
    pub port: u16,
}

impl CmdExecutor for HttpServeOpts {
    async fn execute(self) -> anyhow::Result<()> {
        crate::process_http_serve(self.dir, self.port).await
    }
}

impl CmdExecutor for HttpSubCommand {
    async fn execute(self) -> anyhow::Result<()> {
        match self {
            HttpSubCommand::Serve(opts) => opts.execute().await,
        }
    }
}
