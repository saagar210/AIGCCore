use super::parser::ParsedIncidentEvent;
use super::sanitize::sanitize_untrusted_log;
use crate::error::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Event timeline with metadata and anchors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentTimeline {
    pub timeline_id: String,
    pub incident_id: String,
    pub events: Vec<TimelineEvent>,
    pub total_duration_ms: u64,
    pub high_severity_count: usize,
    pub medium_severity_count: usize,
    pub low_severity_count: usize,
    pub timeline_hash: String,
}

/// Timeline event with anchor for citation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub anchor_id: String,
    pub timestamp_iso: String,
    pub timestamp_epoch_ms: u64,
    pub source_system: String,
    pub actor: String,
    pub action: String,
    pub affected_resource: String,
    pub evidence_text: String,
    pub severity: String,
}

/// Build incident timeline from parsed events
pub fn build_timeline(
    incident_id: &str,
    events: Vec<ParsedIncidentEvent>,
) -> CoreResult<IncidentTimeline> {
    if events.is_empty() {
        return Err(CoreError::InvalidInput(
            "Cannot build timeline from empty events".to_string(),
        ));
    }

    let timeline_id = format!("TIMELINE_{}", incident_id);

    // Calculate duration
    let first_event_ms = events[0].timestamp_epoch_ms;
    let last_event_ms = events[events.len() - 1].timestamp_epoch_ms;
    let total_duration_ms = if last_event_ms > first_event_ms {
        last_event_ms - first_event_ms
    } else {
        0
    };

    // Count by severity
    let high_count = events.iter().filter(|e| e.severity == "HIGH").count();
    let medium_count = events.iter().filter(|e| e.severity == "MEDIUM").count();
    let low_count = events.iter().filter(|e| e.severity == "LOW").count();

    // Convert to timeline events with anchors
    let mut timeline_events = Vec::new();
    let mut hash_input = String::new();

    for event in events {
        // Sanitize evidence text
        let sanitized = sanitize_untrusted_log(&event.evidence_text);

        // Generate deterministic anchor
        let anchor = generate_timeline_anchor(
            incident_id,
            &event.event_id,
            &sanitized.content,
            event.timestamp_epoch_ms,
        );

        // Contribute to timeline hash
        hash_input.push_str(&format!("{}{}", anchor, event.timestamp_epoch_ms));

        timeline_events.push(TimelineEvent {
            anchor_id: anchor,
            timestamp_iso: event.timestamp_iso,
            timestamp_epoch_ms: event.timestamp_epoch_ms,
            source_system: event.source_system,
            actor: event.actor,
            action: event.action,
            affected_resource: event.affected_resource,
            evidence_text: sanitized.content,
            severity: event.severity,
        });
    }

    // Compute timeline hash
    let timeline_hash = compute_timeline_hash(&hash_input);

    Ok(IncidentTimeline {
        timeline_id,
        incident_id: incident_id.to_string(),
        events: timeline_events,
        total_duration_ms,
        high_severity_count: high_count,
        medium_severity_count: medium_count,
        low_severity_count: low_count,
        timeline_hash,
    })
}

/// Generate deterministic anchor for timeline event
fn generate_timeline_anchor(
    incident_id: &str,
    event_id: &str,
    evidence_text: &str,
    timestamp_ms: u64,
) -> String {
    let combined = format!("{}{}{}{}", incident_id, event_id, evidence_text, timestamp_ms);

    let mut hasher = Sha256::new();
    hasher.update(combined.as_bytes());
    let hash_bytes = hasher.finalize();
    let hash_hex = hex::encode(&hash_bytes[0..8]); // Use first 8 bytes for brevity

    format!("INCIDENT_{}_{}", event_id, hash_hex)
}

/// Compute deterministic hash for entire timeline
fn compute_timeline_hash(hash_input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(hash_input.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// Render timeline as CSV format
pub fn render_timeline_csv(timeline: &IncidentTimeline) -> CoreResult<String> {
    let mut csv = String::from("timestamp,system,actor,action,resource,severity,anchor_id\n");

    for event in &timeline.events {
        // CSV escape evidence text
        let resource_csv = event.affected_resource.replace('"', "\"\"");
        csv.push_str(&format!(
            "{},{},{},{},{},{},{}\n",
            event.timestamp_iso,
            event.source_system,
            event.actor,
            event.action,
            resource_csv,
            event.severity,
            event.anchor_id
        ));
    }

    Ok(csv)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_event(idx: u32, timestamp: u64, severity: &str) -> ParsedIncidentEvent {
        ParsedIncidentEvent {
            event_id: format!("EVT_{:04x}", idx),
            timestamp_epoch_ms: timestamp,
            timestamp_iso: "2026-02-12T10:15:30Z".to_string(),
            source_system: "web-server".to_string(),
            actor: format!("user{}", idx),
            action: "login".to_string(),
            affected_resource: "auth-service".to_string(),
            evidence_text: format!("Event {}", idx),
            severity: severity.to_string(),
        }
    }

    #[test]
    fn test_build_timeline_basic() {
        let events = vec![
            sample_event(1, 1000, "LOW"),
            sample_event(2, 2000, "MEDIUM"),
            sample_event(3, 3000, "HIGH"),
        ];

        let timeline = build_timeline("incident_001", events).unwrap();
        assert_eq!(timeline.events.len(), 3);
        assert_eq!(timeline.high_severity_count, 1);
        assert_eq!(timeline.medium_severity_count, 1);
        assert_eq!(timeline.low_severity_count, 1);
    }

    #[test]
    fn test_timeline_duration() {
        let events = vec![
            sample_event(1, 1000, "LOW"),
            sample_event(2, 5000, "LOW"),
        ];

        let timeline = build_timeline("incident_001", events).unwrap();
        assert_eq!(timeline.total_duration_ms, 4000);
    }

    #[test]
    fn test_timeline_anchor_generation() {
        let events = vec![sample_event(1, 1000, "LOW")];
        let timeline = build_timeline("incident_001", events).unwrap();

        assert!(!timeline.events[0].anchor_id.is_empty());
        assert!(timeline.events[0].anchor_id.starts_with("INCIDENT_"));
    }

    #[test]
    fn test_timeline_determinism() {
        let events = vec![
            sample_event(1, 1000, "LOW"),
            sample_event(2, 2000, "HIGH"),
        ];

        let timeline1 = build_timeline("incident_001", events.clone()).unwrap();
        let timeline2 = build_timeline("incident_001", events).unwrap();

        // Same events should produce same hash
        assert_eq!(timeline1.timeline_hash, timeline2.timeline_hash);
    }

    #[test]
    fn test_render_timeline_csv() {
        let events = vec![sample_event(1, 1000, "HIGH")];
        let timeline = build_timeline("incident_001", events).unwrap();

        let csv = render_timeline_csv(&timeline).unwrap();
        assert!(csv.contains("timestamp,system,actor"));
        assert!(csv.contains("web-server"));
        assert!(csv.contains("HIGH"));
    }

    #[test]
    fn test_empty_timeline_fails() {
        let events: Vec<ParsedIncidentEvent> = Vec::new();
        let result = build_timeline("incident_001", events);
        assert!(result.is_err());
    }
}
