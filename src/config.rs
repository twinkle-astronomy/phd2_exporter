use prometheus_exporter::prometheus::Error;
use prometheus_exporter::prometheus::{exponential_buckets, linear_buckets};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum HistogramBuckets {
    Linear {
        start: f64,
        width: f64,
        count: usize,
    },
    Exponential {
        start: f64,
        factor: f64,
        count: usize,
    },
    List(Vec<f64>),
}

impl TryFrom<HistogramBuckets> for Vec<f64> {
    type Error = Error;

    fn try_from(value: HistogramBuckets) -> std::result::Result<Self, Self::Error> {
        match value {
            HistogramBuckets::Linear {
                start,
                width,
                count,
            } => linear_buckets(start, width, count),
            HistogramBuckets::Exponential {
                start,
                factor,
                count,
            } => exponential_buckets(start, factor, count),
            HistogramBuckets::List(v) => Ok(v),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Metrics {
    pub guide_snr_histo: HistogramBuckets,
    pub guide_star_mass_histo: HistogramBuckets,
    pub guide_hfd_histo: HistogramBuckets,
    pub total_distance_raw_histo: HistogramBuckets,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            guide_snr_histo: HistogramBuckets::Linear {
                start: 10.0,
                width: 5.0,
                count: 50,
            },
            guide_star_mass_histo: HistogramBuckets::Exponential {
                start: 10_000.0,
                factor: 1.1,
                count: 50,
            },
            guide_hfd_histo: HistogramBuckets::Linear {
                start: 1.0,
                width: 0.1,
                count: 50,
            },
            total_distance_raw_histo: HistogramBuckets::Exponential {
                start: 0.01,
                factor: 1.1,
                count: 100,
            }
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Server {
    pub address: String,
    pub listen: String,
}

impl Default for Server {
    fn default() -> Self {
        Server {
            address: String::from("localhost:4400"),
            listen: String::from("0.0.0.0:9187"),
        }
    }
}
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Config {
    pub server: Server,
    pub metrics: Metrics,
}
