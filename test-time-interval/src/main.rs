use chrono::{DateTime, Duration, TimeZone, Utc};
use tokio::time::{interval_at, Instant};
use tracing::info;

const STATISTICS_INTERVAL: i64 = 600;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let diff = next_tick_after();
    info!("next tick after = {diff} seconds");
    let start = Instant::now() + tokio::time::Duration::from_secs(diff);
    let mut interval = interval_at(
        start,
        tokio::time::Duration::from_secs(u64::try_from(STATISTICS_INTERVAL).unwrap_or(600)),
    );
    loop {
        interval.tick().await;
        let now = Utc::now();
        info!("correcting... = {now}");
        let corrected = correct_time(now);
        info!("corrected... = {corrected}");
    }
}

fn next_tick_after() -> u64 {
    let now = Utc::now();
    if let Some(after_10min) = now.checked_add_signed(Duration::seconds(STATISTICS_INTERVAL)) {
        let diff = STATISTICS_INTERVAL - after_10min.timestamp() % STATISTICS_INTERVAL;
        return u64::try_from(diff).unwrap_or_default();
    }
    0
}

fn correct_time(now: DateTime<Utc>) -> DateTime<Utc> {
    if let Some(no_nanos) = Utc.timestamp_opt(now.timestamp() + 10, 0).latest() {
        // 10: 10초의 오차 보정 가능
        let diff = no_nanos.timestamp() % STATISTICS_INTERVAL;
        info!("remove nanos = {no_nanos}, diff = {diff}");
        if let Some(corrected) = no_nanos.checked_sub_signed(Duration::seconds(diff)) {
            return corrected;
        }
    }
    now
}
