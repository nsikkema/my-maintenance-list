use rand::random;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Clone, PartialEq)]
enum EventType {
    Info,
    Warning,
    Error,
}

#[derive(Serialize, Clone)]
pub(crate) struct Event {
    #[serde(rename = "type")]
    event_type: EventType,

    #[serde(flatten)]
    keys: HashMap<String, String>,
}

#[derive(Serialize, Clone)]
pub struct WideEvent {
    #[serde(skip_serializing_if = "Option::is_none")]
    service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    deployment_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    region: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    request_id: Option<String>,

    timestamp: String,
    method: String,
    path: String,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    events: Vec<Event>,
    contains_error: bool,

    pub(crate) status_code: u16,
    pub(crate) duration_ms: u64,

    #[serde(skip_serializing)]
    force_log: bool,
}

impl WideEvent {
    pub(crate) fn new(
        request_id: Option<String>,
        method: String,
        path: String,
        service: Option<String>,
        version: Option<String>,
        deployment_id: Option<String>,
        region: Option<String>,
    ) -> Self {
        Self {
            service,
            version,
            deployment_id,
            region,

            request_id,

            timestamp: chrono::Utc::now().to_rfc3339(),
            method,
            path,

            events: Vec::new(),

            status_code: 0,
            duration_ms: 0,
            contains_error: false,

            force_log: false,
        }
    }

    pub fn info(&mut self, keys: HashMap<String, String>) {
        self.events.push(Event {
            event_type: EventType::Info,
            keys,
        });
    }

    pub fn warning(&mut self, keys: HashMap<String, String>) {
        self.events.push(Event {
            event_type: EventType::Warning,
            keys,
        });
    }

    pub fn error(&mut self, keys: HashMap<String, String>) {
        self.events.push(Event {
            event_type: EventType::Error,
            keys,
        });
        self.contains_error = true;
    }

    pub fn force_log(&mut self) {
        self.force_log = true;
    }

    pub(crate) fn should_sample(&mut self, sampling_enabled: bool) -> bool {
        !sampling_enabled
            || self.force_log
            || self.status_code >= 500
            || self.contains_error
            || self.duration_ms > 2000
            || random::<f64>() < 0.05
    }
}
