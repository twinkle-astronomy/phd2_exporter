use log::debug;
use phd2::{
    serialization::{Event, ServerEvent},
    Phd2Connection,
};
use prometheus_exporter::prometheus::{
    exponential_buckets, histogram_opts, linear_buckets, opts, register_gauge_vec,
    register_histogram_vec,
};

pub struct Metrics {
    pub connected: prometheus_exporter::prometheus::GaugeVec,

    // guide_distance: GenericGaugeVec<AtomicF64>,
    guide_snr: prometheus_exporter::prometheus::GaugeVec,
    guide_snr_histo: prometheus_exporter::prometheus::HistogramVec,

    guide_star_mass: prometheus_exporter::prometheus::GaugeVec,
    guide_star_mass_histo: prometheus_exporter::prometheus::HistogramVec,

    guide_hfd: prometheus_exporter::prometheus::GaugeVec,
    guide_hfd_histo: prometheus_exporter::prometheus::HistogramVec,

    ra_distance_raw: prometheus_exporter::prometheus::GaugeVec,
    de_distance_raw: prometheus_exporter::prometheus::GaugeVec,

    total_distance_raw: prometheus_exporter::prometheus::GaugeVec,
    total_distance_raw_histo: prometheus_exporter::prometheus::HistogramVec,

    is_dithering_metric: prometheus_exporter::prometheus::GaugeVec,
    is_settling_metric: prometheus_exporter::prometheus::GaugeVec,

    pixel_scale: prometheus_exporter::prometheus::GaugeVec,

    is_dithering: bool,
    is_settling: bool,
}

impl Metrics {
    pub fn new() -> Self {
        let connected = register_gauge_vec!(opts!("phd2_connected", "Status of connection to phd2"), &[]).unwrap();
        let guide_snr = register_gauge_vec!(
            opts!(
                "phd2_guide_snr",
                "Guide snr"
            ),
            &["host", "mount", "is_dithering", "is_settling",]
        )
        .unwrap();

        let guide_snr_histo = register_histogram_vec!(
            histogram_opts!(
                "phd2_guide_snr_histo",
                "Histogram of snr",
                linear_buckets(10.0, 5.0, 50).unwrap()
            ),
            &["host", "mount", "is_dithering", "is_settling",]
        )
        .unwrap();

        let guide_star_mass = register_gauge_vec!(
            opts!("phd2_guide_star_mass", "Guide star_mass"),
            &["host", "mount", "is_dithering", "is_settling",]
        )
        .unwrap();

        let guide_star_mass_histo = register_histogram_vec!(
            histogram_opts!(
                "phd2_guide_star_mass_histo",
                "Histogram of guid star mass",
                exponential_buckets(10_000.0, 1.1, 50).unwrap()
            ),
            &["host", "mount", "is_dithering", "is_settling",]
        )
        .unwrap();

        let guide_hfd = register_gauge_vec!(
            opts!("phd2_guide_hfd", "Guide star_mass"),
            &["host", "mount", "is_dithering", "is_settling",]
        )
        .unwrap();

        let guide_hfd_histo = register_histogram_vec!(
            histogram_opts!(
                "phd2_guide_hfd_histo",
                "Histogram of guide hfd",
                linear_buckets(1.0, 0.1, 50).unwrap()
            ),
            &["host", "mount", "is_dithering", "is_settling",]
        )
        .unwrap();

        let ra_distance_raw = register_gauge_vec!(
            opts!(
                "phd2_ra_distance_raw",
                "The RA distance in pixels of the guide offset vector"
            ),
            &["host", "mount", "is_dithering", "is_settling",]
        )
        .unwrap();

        let de_distance_raw = register_gauge_vec!(
            opts!(
                "phd2_de_distance_raw",
                "The DEC distance in pixels of the guide offset vector"
            ),
            &["host", "mount", "is_dithering", "is_settling",]
        )
        .unwrap();

        let total_distance_raw = register_gauge_vec!(
            opts!(
                "phd2_total_distance_raw",
                "The total distance in pixels of the guide offset vector"
            ),
            &["host", "mount", "is_dithering", "is_settling",]
        )
        .unwrap();

        let total_distance_raw_histo = register_histogram_vec!(
            histogram_opts!(
                "phd2_total_distance_raw_histo",
                "Histogram of the total distance in pixels of the guide offset vector",
                exponential_buckets(0.01, 1.1, 100).unwrap()
            ),
            &["host", "mount", "is_dithering", "is_settling",]
        )
        .unwrap();

        let is_dithering_metric = register_gauge_vec!(
            opts!(
                "phd2_is_dithering",
                "Boolean indicating if PHD2 is currently dithering",
            ),
            &["host",]
        )
        .unwrap();

        let is_settling_metric = register_gauge_vec!(
            opts!(
                "phd2_is_settling",
                "Boolean indicating if PHD2 is currently settling",
            ),
            &["host",]
        )
        .unwrap();

        let pixel_scale = register_gauge_vec!(
            opts!("phd2_pixel_scale", "Guider image scale in arc-sec/pixel."),
            &[]
        )
        .unwrap();

        Metrics {
            connected,
            guide_snr,
            guide_snr_histo,
            guide_star_mass,
            guide_star_mass_histo,
            guide_hfd,
            guide_hfd_histo,
            ra_distance_raw,
            de_distance_raw,    
            total_distance_raw,
            total_distance_raw_histo,
            is_dithering_metric,
            is_settling_metric,
            pixel_scale,
            is_dithering: false,
            is_settling: false,
        }
    }

    fn handle_event(&mut self, event: &ServerEvent) {
        match &event.event {
            Event::GuideStep(guide) => {
                let is_dithering = match &self.is_dithering {
                    false => "0",
                    true => "1",
                };

                let is_settling = match &self.is_settling {
                    false => "0",
                    true => "1",
                };

                let snr = guide.snr;
                // dbg!(snr);
                self.guide_snr
                    .with_label_values(&[&event.host, &guide.mount, is_dithering, is_settling])
                    .set(snr);

                self.guide_snr_histo
                    .with_label_values(&[&event.host, &guide.mount, is_dithering, is_settling])
                    .observe(snr);

                let star_mass = guide.star_mass;
                // dbg!(star_mass);
                self.guide_star_mass
                    .with_label_values(&[&event.host, &guide.mount, is_dithering, is_settling])
                    .set(star_mass);

                self.guide_star_mass_histo
                    .with_label_values(&[&event.host, &guide.mount, is_dithering, is_settling])
                    .observe(star_mass);

                let hfd = guide.hfd;
                // dbg!(hfd);
                self.guide_hfd
                    .with_label_values(&[&event.host, &guide.mount, is_dithering, is_settling])
                    .set(hfd);

                self.guide_hfd_histo
                    .with_label_values(&[&event.host, &guide.mount, is_dithering, is_settling])
                    .observe(hfd);

                let ra_distance_raw = guide.ra_distance_raw;
                self.ra_distance_raw
                    .with_label_values(&[&event.host, &guide.mount, is_dithering, is_settling])
                    .set(ra_distance_raw);

                let de_distance_raw = guide.de_distance_raw;
                self.de_distance_raw
                    .with_label_values(&[&event.host, &guide.mount, is_dithering, is_settling])
                    .set(de_distance_raw);

                let total_distance_raw = (guide.ra_distance_raw * guide.ra_distance_raw
                    + guide.de_distance_raw * guide.de_distance_raw)
                    .sqrt();
                self.total_distance_raw
                    .with_label_values(&[&event.host, &guide.mount, is_dithering, is_settling])
                    .set(total_distance_raw);

                self.total_distance_raw_histo
                    .with_label_values(&[&event.host, &guide.mount, is_dithering, is_settling])
                    .observe(total_distance_raw);
            },
            Event::GuidingDithered(_) => {
                self.is_dithering = true;
                self.is_dithering_metric
                    .with_label_values(&[&event.host])
                    .set(1.0);
            },
            Event::SettleBegin(_) => {
                self.is_settling = true;
                self.is_settling_metric
                    .with_label_values(&[&event.host])
                    .set(1.0);
            },
            Event::Settling(_) => {
                self.is_settling_metric
                    .with_label_values(&[&event.host])
                    .set(1.0);
            },
            Event::SettleDone(_) => {
                self.is_dithering = false;
                self.is_settling = false;
                self.is_dithering_metric
                    .with_label_values(&[&event.host])
                    .set(0.0);
                self.is_settling_metric
                    .with_label_values(&[&event.host])
                    .set(0.0);
            },
            _ => {}
        }
    }

    pub async fn async_run<T: Send + tokio::io::AsyncRead + tokio::io::AsyncWrite>(
        &mut self,
        connection: Phd2Connection<T>,
        mut recv: tokio::sync::mpsc::Receiver<ServerEvent>,
    ) -> () {
        let mut seen_first_event: bool = false;
        
        if let Ok(scale) = connection.get_pixel_scale().await {
            self.pixel_scale.with_label_values(&[]).set(scale);
        }

        loop {
            let event = recv.recv().await;
            debug!(target: "phd2_events", "{:?}", &event);
            match event {
                Some(event) => {
                    if !seen_first_event {
                        self.is_settling_metric
                            .with_label_values(&[&event.host])
                            .set(0.0);
                        seen_first_event = true;
                    }

                    self.handle_event(&event);

                    match &event.event {
                        Event::ConfigurationChange(_) => {
                            if let Ok(scale) = connection.get_pixel_scale().await {
                                self.pixel_scale.with_label_values(&[]).set(scale);
                            }
                        }
                        _ => {}
                    }
                }
                None => return (),
            }
        }
    }
}
