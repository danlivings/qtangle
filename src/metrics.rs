use opentelemetry::metrics::Meter;
use opentelemetry::{KeyValue, global};
use std::time::Duration;

const METER_NAME: &str = "qtangle";

const INSTRUMENT_NAME_POLLING_INTERVAL: &str = "config.polling_interval";
const INSTRUMENT_DESC_POLLING_INTERVAL: &str = "How often Qtangle will poll the QBittorrent API.";

const INSTRUMENT_NAME_MAX_CONCURRENCY: &str = "config.max_concurrency";
const INSTRUMENT_DESC_MAX_CONCURRENCY: &str =
    "The maximum number of simultaneous filesystem operations Qtangle will perform.";

const INSTRUMENT_NAME_SEED_RATIO: &str = "config.seed_ratio";
const INSTRUMENT_DESC_SEED_RATIO: &str =
    "The maximum seed ratio before Qtangle will instruct QBittorrent to delete a torrent.";

const INSTRUMENT_NAME_TORRENTS_COMPLETED: &str = "engine.torrents_completed";
const INSTRUMENT_DESC_TORRENTS_COMPLETED: &str = "The number of torrents that have finished downloading and are ready to be processed by Qtangle.";

const INSTRUMENT_NAME_TORRENTS_PROCESSED: &str = "engine.torrents_processed";
const INSTRUMENT_DESC_TORRENTS_PROCESSED: &str =
    "The total number of torrents that Qtangle has processed.";

const INSTRUMENT_NAME_TORRENTS_DELETED: &str = "engine.torrents_deleted";
const INSTRUMENT_DESC_TORRENTS_DELETED: &str =
    "The total number of torrents that Qtangle has deleted after hitting the maximum seed ratio.";

const INSTRUMENT_NAME_BYTES_COPIED: &str = "engine.bytes_copied";
const INSTRUMENT_DESC_BYTES_COPIED: &str = "The total number of bytes that Qtangle has copied.";

const INSTRUMENT_NAME_FILESYSTEM_OPERATIONS: &str = "engine.filesystem_operations";
const INSTRUMENT_DESC_FILESYSTEM_OPERATIONS: &str =
    "The number of filesystem operations currently being performed.";

const INSTRUMENT_NAME_FILESYSTEM_SUCCESSES: &str = "engine.filesystem_successes";
const INSTRUMENT_DESC_FILESYSTEM_SUCCESSES: &str =
    "The total number of filesystem operations that have succeeded.";

const INSTRUMENT_NAME_FILESYSTEM_ERRORS: &str = "engine.filesystem_errors";
const INSTRUMENT_DESC_FILESYSTEM_ERRORS: &str =
    "The total number of errors Qtangle has encountered while performing filesystem operations.";

fn meter() -> Meter {
    global::meter(METER_NAME)
}

pub fn set_metric_polling_interval(duration: Duration) {
    let gauge = meter()
        .f64_gauge(INSTRUMENT_NAME_POLLING_INTERVAL)
        .with_description(INSTRUMENT_DESC_POLLING_INTERVAL)
        .build();
    gauge.record(duration.as_secs_f64(), &[]);
}

pub fn set_metric_max_concurrency(max_concurrency: usize) {
    let gauge = meter()
        .u64_gauge(INSTRUMENT_NAME_MAX_CONCURRENCY)
        .with_description(INSTRUMENT_DESC_MAX_CONCURRENCY)
        .build();
    gauge.record(max_concurrency as u64, &[]);
}

pub fn set_metric_seed_ratio(seed_ratio: f64) {
    let gauge = meter()
        .f64_gauge(INSTRUMENT_NAME_SEED_RATIO)
        .with_description(INSTRUMENT_DESC_SEED_RATIO)
        .build();
    gauge.record(seed_ratio, &[]);
}

pub fn set_metric_torrents_completed(count: u64, tag: &str) {
    let gauge = meter()
        .u64_gauge(INSTRUMENT_NAME_TORRENTS_COMPLETED)
        .with_description(INSTRUMENT_DESC_TORRENTS_COMPLETED)
        .build();
    gauge.record(count, &[KeyValue::new("tag", tag.to_string())]);
}

pub fn increment_metric_torrents_processed(tag: &str) {
    let counter = meter()
        .u64_counter(INSTRUMENT_NAME_TORRENTS_PROCESSED)
        .with_description(INSTRUMENT_DESC_TORRENTS_PROCESSED)
        .build();
    counter.add(1, &[KeyValue::new("tag", tag.to_string())]);
}

pub fn increment_metric_torrents_deleted(tag: &str) {
    let counter = meter()
        .u64_counter(INSTRUMENT_NAME_TORRENTS_DELETED)
        .with_description(INSTRUMENT_DESC_TORRENTS_DELETED)
        .build();
    counter.add(1, &[KeyValue::new("tag", tag.to_string())]);
}

pub fn add_metric_bytes_copied(bytes: u64) {
    let counter = meter()
        .u64_counter(INSTRUMENT_NAME_BYTES_COPIED)
        .with_description(INSTRUMENT_DESC_BYTES_COPIED)
        .build();
    counter.add(bytes, &[]);
}

pub fn increment_metric_filesystem_operations(task_type: &str) {
    let counter = meter()
        .i64_up_down_counter(INSTRUMENT_NAME_FILESYSTEM_OPERATIONS)
        .with_description(INSTRUMENT_DESC_FILESYSTEM_OPERATIONS)
        .build();
    counter.add(1, &[KeyValue::new("task_type", task_type.to_string())]);
}

pub fn decrement_metric_filesystem_operations(task_type: &str) {
    let counter = meter()
        .i64_up_down_counter(INSTRUMENT_NAME_FILESYSTEM_OPERATIONS)
        .with_description(INSTRUMENT_DESC_FILESYSTEM_OPERATIONS)
        .build();
    counter.add(-1, &[KeyValue::new("task_type", task_type.to_string())]);
}

pub fn increment_metric_filesystem_successes(task_type: &str) {
    let counter = meter()
        .u64_counter(INSTRUMENT_NAME_FILESYSTEM_SUCCESSES)
        .with_description(INSTRUMENT_DESC_FILESYSTEM_SUCCESSES)
        .build();
    counter.add(1, &[KeyValue::new("task_type", task_type.to_string())]);
}

pub fn increment_metric_filesystem_errors(task_type: &str) {
    let counter = meter()
        .u64_counter(INSTRUMENT_NAME_FILESYSTEM_ERRORS)
        .with_description(INSTRUMENT_DESC_FILESYSTEM_ERRORS)
        .build();
    counter.add(1, &[KeyValue::new("task_type", task_type.to_string())]);
}
