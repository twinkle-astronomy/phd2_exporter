mod config;
mod metrics;
use std::{fs::File, time::Duration};

use log::{error, info, warn};
use metrics::Metrics;

use clap::Parser;
use phd2::Phd2Connection;
use tokio::net::TcpStream;

#[derive(Parser, Debug)]
struct Args {
    /// Filename of configuration file.
    #[arg(default_value = None)]
    config: Option<String>,
}

async fn run_loop(config: &config::Config, metrics: &Metrics) {
    let connection = match TcpStream::connect(&config.server.address).await {
        Ok(c) => c,
        Err(e) => {
            error!("Error connecting to phd2: {}", e);
            return;
        }
    };

    metrics.connected.with_label_values(&[]).set(1.0);
    let (phd2, events): (Phd2Connection<_>, _) = Phd2Connection::from(connection);
    metrics.async_run(phd2, events).await;
}

#[tokio::main]
async fn main() {
    env_logger::builder().parse_env("LOG").init();

    let config_filename = Args::parse().config;
    let config = match &config_filename {
        Some(filename) => {
            let file = File::open(filename);
            match file {
                Ok(file) => serde_yaml::from_reader(file).expect("Reading config file"),
                Err(err) => {
                    warn!(
                        "Unable to open config file, attempting to create one: {:?}",
                        err
                    );
                    let file: File = File::create(filename).expect("Creating default config file.");
                    let config: config::Config = Default::default();
                    serde_yaml::to_writer(file, &config).expect("Writing config file.");
                    config
                }
            }
        }
        None => Default::default(),
    };

    prometheus_exporter::start(config.server.listen.parse().unwrap())
        .expect("Starting prometheus server");

    let metrics = Metrics::new(&config.metrics);
    loop {
        info!("Connecting to {}", config.server.address);
        run_loop(&config, &metrics).await;
        metrics.connected.with_label_values(&[]).set(0.0);

        info!("Disconnected, waiting 1s to reconnect");
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
