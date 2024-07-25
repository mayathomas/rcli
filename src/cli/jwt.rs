use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use clap::Parser;
use enum_dispatch::enum_dispatch;

use crate::{proceess_jwt_sign, process_jwt_verify, CmdExecutor};

#[derive(Debug, Parser)]
#[enum_dispatch(CmdExecutor)]
pub enum JwtSubCommand {
    #[command(about = "Sign a message with a private key")]
    Sign(JwtSignOpts),
    #[command(about = "Verify a signed message")]
    Verify(JwtVerifyOpts),
}

#[derive(Debug, Parser)]
pub struct JwtSignOpts {
    #[arg(long)]
    pub sub: String,
    #[arg(long)]
    pub aud: String,
    #[arg(long, value_parser = verify_expiration)]
    pub exp: u64,
}

#[derive(Debug, Parser)]
pub struct JwtVerifyOpts {
    #[arg(short, long)]
    pub token: String,
}

impl CmdExecutor for JwtSignOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let token = proceess_jwt_sign(self.sub, self.aud, self.exp)?;
        println!("token: {}", token);
        Ok(())
    }
}

impl CmdExecutor for JwtVerifyOpts {
    async fn execute(self) -> anyhow::Result<()> {
        process_jwt_verify(self.token)?;
        println!("verify success!");
        Ok(())
    }
}

fn verify_expiration(exp: &str) -> Result<u64> {
    if exp.ends_with('d') || exp.ends_with('D') {
        let days = &exp[0..exp.len() - 1];
        let days = days.parse::<u64>().unwrap();
        let now = SystemTime::now();
        let duration = Duration::from_secs(days * 86400);
        let timestamp = now + duration;
        let timestamp = timestamp
            .duration_since(UNIX_EPOCH)
            .map(|dur| dur.as_secs())?;
        Ok(timestamp)
    } else {
        match exp.parse::<u64>() {
            Ok(exp) => Ok(exp),
            Err(e) => Err(anyhow::anyhow!(e)),
        }
    }
}
