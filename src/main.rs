use std::fs;

use clap::Parser;
use zxcvbn::zxcvbn;

use rcli::{
    process_csv, process_decode, process_encode, process_genpass, process_text_decrypt,
    process_text_encrypt, process_text_generate, process_text_sign, process_text_verify,
    Base64SubCommand, Opts, SubCommand, TextSignFormat, TextSubCommand,
};

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();
    match opts.cmd {
        SubCommand::Csv(opts) => {
            let output = if let Some(output) = opts.output {
                output
            } else {
                //要使用format!宏来转换，就要实现Display Trait
                format!("output.{}", opts.format)
            };
            process_csv(&opts.input, output, opts.format)?;
        }
        SubCommand::GenPass(opts) => {
            let password = process_genpass(
                opts.length,
                opts.uppercase,
                opts.lowercase,
                opts.number,
                opts.symbol,
            )?;
            println!("{}", password);
            //检查密码复杂度
            let estimate = zxcvbn(&password, &[]);
            eprintln!("Password strength: {}", estimate.score());
        }
        SubCommand::Base64(subcmd) => match subcmd {
            Base64SubCommand::Encode(opts) => {
                let encoded = process_encode(&opts.input, opts.format)?;
                println!("{}", encoded);
            }
            Base64SubCommand::Decode(opts) => {
                let decoded = process_decode(&opts.input, opts.format)?;
                println!("{}", String::from_utf8(decoded)?);
            }
        },
        SubCommand::Text(subcmd) => match subcmd {
            TextSubCommand::Sign(opts) => {
                let sig = process_text_sign(&opts.input, &opts.key, opts.format)?;
                print!("{}", sig)
            }
            TextSubCommand::Verify(opts) => {
                let verified = process_text_verify(&opts.input, &opts.key, opts.format, &opts.sig)?;
                println!("{}", verified);
            }
            TextSubCommand::Generate(opts) => {
                let key = process_text_generate(opts.format)?;
                match opts.format {
                    TextSignFormat::Blake3 => {
                        let name = opts.output.join("blake3.txt");
                        fs::write(name, &key[0])?;
                    }
                    TextSignFormat::Ed25519 => {
                        let name = opts.output;
                        fs::write(name.join("ed25519.sk"), &key[0])?;
                        fs::write(name.join("ed25519.pk"), &key[1])?;
                    }
                }
            }
            TextSubCommand::Encrypt(opts) => {
                let encrypted =
                    process_text_encrypt(&opts.input, &opts.key, &opts.nonce, opts.format)?;
                println!("{}", encrypted);
            }
            TextSubCommand::Decrypt(opts) => {
                let decrypted =
                    process_text_decrypt(&opts.input, &opts.key, &opts.nonce, opts.format)?;
                println!("{}", decrypted);
            }
        },
    }
    Ok(())
}

#[cfg(test)]
mod test {
    #[test]
    fn ttt() {}
}
