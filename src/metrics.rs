use prometheus::{Encoder, IntCounterVec, TextEncoder, register_int_counter_vec};

lazy_static::lazy_static! {
    pub static ref REDIRECT_TOTAL: IntCounterVec = register_int_counter_vec!(
        "redirect_total",
        "Total number of redirects",
        &["device", "host"]
    )
    .unwrap();
}

pub fn encode_metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();

    let mut buf = Vec::new();
    encoder.encode(&metric_families, &mut buf).unwrap();

    String::from_utf8(buf).unwrap()
}
