//! WAL → OpenTelemetry SpanData → OTLP HTTP/protobuf export.

use std::collections::BTreeMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use opentelemetry::trace::{SpanContext, SpanId, SpanKind, TraceFlags, TraceId, TraceState};
use opentelemetry::KeyValue;
use opentelemetry_sdk::export::trace::{SpanData, SpanExporter};
use opentelemetry_sdk::trace::{SpanEvents, SpanLinks};
use serde::Deserialize;

use crate::config::ExporterConfig;

#[derive(Debug, Clone, Deserialize)]
pub struct WalLine {
    pub ts: String,
    pub span: String,
    #[serde(flatten)]
    pub attributes: BTreeMap<String, String>,
}

/// Parse WAL JSONL lines into OTel `SpanData` (one span per WAL line, shared trace).
pub fn wal_lines_to_span_data(run_id: &str, lines: &[WalLine]) -> Vec<SpanData> {
    let trace_id = trace_id_from_run(run_id);
    lines
        .iter()
        .enumerate()
        .map(|(idx, line)| wal_line_to_span_data(trace_id, idx, line))
        .collect()
}

pub fn wal_line_to_span_data(trace_id: TraceId, line_index: usize, line: &WalLine) -> SpanData {
    let span_id = span_id_from_index(line_index);
    let start = parse_wal_ts(&line.ts).unwrap_or_else(SystemTime::now);
    let end = start + Duration::from_millis(1);
    let mut attrs: Vec<KeyValue> = line
        .attributes
        .iter()
        .map(|(k, v)| KeyValue::new(k.clone(), v.clone()))
        .collect();
    attrs.push(KeyValue::new("popsicle.wal_index", line_index as i64));

    SpanData {
        span_context: SpanContext::new(
            trace_id,
            span_id,
            TraceFlags::SAMPLED,
            false,
            TraceState::NONE,
        ),
        parent_span_id: if line_index == 0 {
            SpanId::INVALID
        } else {
            span_id_from_index(line_index - 1)
        },
        span_kind: SpanKind::Internal,
        name: line.span.clone().into(),
        start_time: start,
        end_time: end,
        attributes: attrs,
        dropped_attributes_count: 0,
        events: SpanEvents::default(),
        links: SpanLinks::default(),
        status: opentelemetry::trace::Status::Unset,
        instrumentation_scope: opentelemetry::InstrumentationScope::builder("popsicle-telemetry")
            .build(),
    }
}

pub fn trace_id_from_run(run_id: &str) -> TraceId {
    let hex: String = run_id
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .take(32)
        .collect();
    if hex.len() == 32 {
        if let Ok(bytes) = (0..16)
            .map(|i| u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16))
            .collect::<Result<Vec<_>, _>>()
        {
            let mut arr = [0u8; 16];
            arr.copy_from_slice(&bytes);
            return TraceId::from_bytes(arr);
        }
    }
    TraceId::from_bytes(hash_bytes(run_id, 16))
}

fn span_id_from_index(index: usize) -> SpanId {
    let full = hash_bytes(&format!("span:{index}"), 8);
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&full[..8]);
    SpanId::from_bytes(bytes)
}

fn hash_bytes(seed: &str, len: usize) -> [u8; 16] {
    let mut out = [0u8; 16];
    let bytes = seed.as_bytes();
    for (i, b) in bytes.iter().cycle().take(len).enumerate() {
        out[i % 16] ^= b.wrapping_mul((i as u8).wrapping_add(1));
    }
    out
}

fn parse_wal_ts(ts: &str) -> Option<SystemTime> {
    let ts = ts.strip_suffix('Z')?;
    let (secs, millis) = ts.split_once('.')?;
    let secs: u64 = secs.parse().ok()?;
    let millis: u32 = millis.parse().ok()?;
    Some(UNIX_EPOCH + Duration::from_secs(secs) + Duration::from_millis(millis as u64))
}

pub fn build_span_exporter(
    config: &ExporterConfig,
) -> Result<opentelemetry_otlp::SpanExporter, String> {
    use opentelemetry_otlp::{Protocol, WithExportConfig};

    let protocol = config.protocol.trim();
    if !protocol.is_empty() && protocol != "http/protobuf" {
        return Err(format!(
            "unsupported exporter.protocol: {protocol} (use http/protobuf)"
        ));
    }

    let endpoint = config.endpoint.trim();
    if endpoint.is_empty() {
        return Err("exporter.endpoint required".into());
    }

    let mut builder = opentelemetry_otlp::SpanExporter::builder().with_http();
    builder = builder.with_protocol(Protocol::HttpBinary);
    let exp = builder
        .with_endpoint(endpoint.to_string())
        .build()
        .map_err(|e| e.to_string())?;

    Ok(exp)
}

pub fn export_span_batch(
    exporter: &mut opentelemetry_otlp::SpanExporter,
    batch: Vec<SpanData>,
) -> Result<(), String> {
    if batch.is_empty() {
        return Ok(());
    }
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    rt.block_on(async {
        exporter
            .export(batch)
            .await
            .map_err(|e: opentelemetry::trace::TraceError| e.to_string())
    })
}

pub fn parse_wal_json_lines(raw: &[String]) -> Vec<WalLine> {
    raw.iter()
        .filter_map(|l| serde_json::from_str::<WalLine>(l).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_id_from_uuid_like_run() {
        let id = trace_id_from_run("00000045-0000-4045-8004-45000000000045");
        assert_ne!(id, TraceId::INVALID);
    }

    #[test]
    fn wal_line_maps_to_span_data() {
        let line = WalLine {
            ts: "1000.500Z".into(),
            span: "popsicle.run.start".into(),
            attributes: BTreeMap::from([("popsicle.issue_key".into(), "PROJ-70".into())]),
        };
        let trace = trace_id_from_run("run-1");
        let sd = wal_line_to_span_data(trace, 0, &line);
        assert_eq!(sd.name, "popsicle.run.start");
        assert!(!sd.attributes.is_empty());
    }
}
