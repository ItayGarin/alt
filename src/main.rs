mod error;
mod events;

mod gateway;
mod i3_focus;
mod aggregator;
mod ktrl_client;
mod config;

use error::DynError;
use gateway::EvGateway;
use i3_focus::I3FocusListener;
use aggregator::EvAggregator;
use ktrl_client::KtrlClient;
use config::AltCfg;

use tokio::sync::mpsc;

use dirs::home_dir;
use clap::{App, Arg};
use log::info;
use simplelog::*;
use std::fs::File;
use std::io::{Error, ErrorKind::*};
use std::path::{Path, PathBuf};

const DEFAULT_LOG_PATH: &str = ".alt.log";
const DEFAULT_CFG_PATH: &str = ".alt.ron";

struct AltArgs {
    config_path: PathBuf,
}

fn cli_init() -> Result<AltArgs, std::io::Error> {
    let matches = App::new("alt")
        .version("0.1")
        .author("Itay G. <thifixp@gmail.com>")
        .about("An Event Aggregator for ktrl")
        .arg(
            Arg::with_name("cfg")
                .long("cfg")
                .value_name("CONFIG")
                .help(&format!(
                    "Path to your alt config file. Default: {}",
                    DEFAULT_CFG_PATH
                ))
                .takes_value(true),
        )
        .arg(
            Arg::with_name("logfile")
                .long("log")
                .value_name("LOGFILE")
                .help(&format!(
                    "Path to the log file. Default: {}",
                    DEFAULT_LOG_PATH
                ))
                .takes_value(true),
        )
        .arg(
            Arg::with_name("debug")
                .long("debug")
                .help("Enables debug level logging"),
        )
        .get_matches();

    let home_dir = home_dir().expect("Could not find your home directory");

    let config_path = match matches.value_of("cfg") {
        Some(path) => Path::new(path).to_owned(),
        _ => home_dir.join(DEFAULT_CFG_PATH),
    };

    let log_path = match matches.value_of("logfile") {
        Some(path) => Path::new(path).to_owned(),
        _ => home_dir.join(DEFAULT_LOG_PATH),
    };

    let log_lvl = match matches.is_present("debug") {
        true => LevelFilter::Debug,
        _ => LevelFilter::Info,
    };

    CombinedLogger::init(vec![
        TermLogger::new(log_lvl, Config::default(), TerminalMode::Mixed),
        WriteLogger::new(
            log_lvl,
            Config::default(),
            File::create(log_path).expect("Couldn't initialize the file logger"),
        ),
    ])
    .expect("Couldn't initialize the logger");

    if !config_path.exists() {
        let err = format!(
            "Could not find your config file ({})",
            config_path.to_str().unwrap_or("?")
        );
        return Err(Error::new(NotFound, err));
    }

    Ok(
        AltArgs{
            config_path
        })
}

#[tokio::main]
async fn main() -> Result<(), DynError> {
    let args = cli_init()?;
    let cfg = AltCfg::parse(&args.config_path)?;

    info!("ALT: Started");

    let (agg_tx, agg_rx) = mpsc::channel(1);
    let (ktrl_tx, ktrl_rx) = mpsc::channel(1);

    let gateway = EvGateway::new(agg_tx.clone()).await?;
    let mut i3listener = I3FocusListener::new(agg_tx);
    let mut aggregator = EvAggregator::new(cfg, ktrl_tx, agg_rx);
    let client = KtrlClient::new(ktrl_rx).await?;

    let (gateway_result, i3_res, agg_res, client_res) =
        tokio::join!(
            gateway.event_loop(),
            i3listener.event_loop(),
            aggregator.event_loop(),
            client.event_loop(),
        );

    gateway_result?;
    i3_res?;
    agg_res?;
    client_res?;

    Ok(())
}
