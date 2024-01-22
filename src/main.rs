use mail_builder::MessageBuilder;
use mail_parser::HeaderName;
use mail_parser::HeaderValue;
use mail_parser::*;
use std::fs;
use structopt::StructOpt;

use mail_send::SmtpClientBuilder;

use anyhow::Context;
use anyhow::Result;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str), short = "d", long = "directory")]
    dir: std::path::PathBuf,
    #[structopt(short = "f", long = "from")]
    from: String,
    #[structopt(short = "n", long = "name")]
    from_name: String,
    #[structopt(short = "s", long = "server")]
    server: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::from_args();
    let client = reqwest::Client::new();
    for entry in fs::read_dir(args.dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        let data = fs::read(path)?;
        let message = MessageParser::default()
            .parse(&data)
            .context("Cannot parse message")?;
        let mut maybe_email = None;
        let mut maybe_http = None;
        let mut one_click = None;
        let mut to = None;
        for header in message.headers() {
            if header.name == HeaderName::ListUnsubscribe {
                if let HeaderValue::Address(addresses) = header.value.clone() {
                    for addr in addresses.into_list() {
                        if let Some(addr) = addr.address() {
                            if let Ok(url) = url::Url::parse(addr) {
                                if url.scheme() == "mailto" {
                                    maybe_email = Some(url.path().to_owned());
                                } else if url.scheme() == "https" {
                                    maybe_http = Some(url);
                                }
                            }
                        }
                    }
                }
            }
            if header.name == HeaderName::Other("List-Unsubscribe-Post".into())
            {
                one_click = header.value.as_text().map(|s| s.to_owned());
            }
            if header.name == HeaderName::To {
                if let HeaderValue::Address(addresses) = header.value.clone() {
                    if let Some(first_addr) = addresses.first() {
                        if let Some(addr) = first_addr.address() {
                            to = Some(addr.to_owned());
                        }
                    }
                }
            }
        }
        if to.is_none() {
            to = Some(args.from.clone());
        }
        let to = to.unwrap();
        if let Some(email) = maybe_email {
            let message = MessageBuilder::new()
                .from((args.from_name.clone(), to))
                .to(vec![("Unsubscribe".to_string(), email)])
                .subject("Unsubscribe")
                .html_body("<h1>Unsubscribe!</h1>")
                .text_body("Unsubscribe!");

            SmtpClientBuilder::new(args.server.clone(), 587)
                .implicit_tls(false)
                .connect()
                .await
                .unwrap()
                .send(message)
                .await
                .unwrap();
        }
        if let Some(url) = maybe_http {
            let body =
                one_click.unwrap_or("List-Unsubscribe=One-Click".to_string());
            let _res = client.post(url).body(body).send().await?;
        }
    }
    Ok(())
}
