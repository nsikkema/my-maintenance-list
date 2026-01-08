use axum::{extract::Request, response::Response};
use futures_util::future::BoxFuture;
use rand::random;
use serde::Serialize;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::Instant;
use tower::{Layer, Service};

#[derive(Serialize, Clone, PartialEq)]
enum EventType {
    Info,
    Warning,
    Error,
}

#[derive(Serialize, Clone)]
struct Event {
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

    status_code: u16,
    duration_ms: u64,
    contains_error: bool,

    #[serde(skip_serializing)]
    force_log: bool,
}

impl WideEvent {
    fn new(
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
    }

    pub fn force_log(&mut self) {
        self.force_log = true;
    }

    fn should_sample(&mut self, sampling_enabled: bool) -> bool {
        !sampling_enabled
            || self.force_log
            || self.status_code >= 500
            || self.contains_error
            || self.duration_ms > 2000
            || random::<f64>() < 0.05
    }
}

#[derive(Clone)]
pub struct WideEventLoggerLayer {
    sampling_enabled: bool,
    service: Option<String>,
    version: Option<String>,
    deployment_id: Option<String>,
    region: Option<String>,
}

impl WideEventLoggerLayer {
    pub fn new(
        sampling_enabled: bool,
        service: Option<String>,
        version: Option<String>,
        deployment_id: Option<String>,
        region: Option<String>,
    ) -> Self {
        Self {
            sampling_enabled,
            service,
            version,
            deployment_id,
            region,
        }
    }
}

impl<S> Layer<S> for WideEventLoggerLayer {
    type Service = WideEventLogger<S>;

    fn layer(&self, inner: S) -> Self::Service {
        WideEventLogger {
            inner,
            sampling_enabled: self.sampling_enabled,
            service: self.service.clone(),
            version: self.version.clone(),
            deployment_id: self.deployment_id.clone(),
            region: self.region.clone(),
        }
    }
}

#[derive(Clone)]
pub struct WideEventLogger<S> {
    inner: S,
    sampling_enabled: bool,

    service: Option<String>,
    version: Option<String>,
    deployment_id: Option<String>,
    region: Option<String>,
}

impl<S> Service<Request> for WideEventLogger<S>
where
    S: Service<Request, Response = Response> + Send + Clone + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request) -> Self::Future {
        let sampling_enabled = self.sampling_enabled;
        let start = Instant::now();

        let request_id = request
            .headers()
            .get("x-request-id")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        let method = request.method().to_string();
        let path = request.uri().path().to_string();

        let event = Arc::new(Mutex::new(WideEvent::new(
            request_id,
            method,
            path,
            self.service.clone(),
            self.version.clone(),
            self.deployment_id.clone(),
            self.region.clone(),
        )));

        request.extensions_mut().insert(Arc::clone(&event));

        let mut inner = self.inner.clone();

        Box::pin(async move {
            let response = inner.call(request).await?;

            let status = response.status();
            let mut event_lock = event.lock().expect("Failed to lock wide event");

            event_lock.status_code = status.as_u16();
            event_lock.duration_ms = start.elapsed().as_millis().try_into().unwrap_or(u64::MAX);

            event_lock.contains_error = {
                let mut contains_error = false;
                for event in &event_lock.events {
                    if event.event_type == EventType::Error {
                        contains_error = true;
                    }
                }
                contains_error
            };

            if event_lock.should_sample(sampling_enabled) {
                println!(
                    "{}",
                    serde_json::to_string(&*event_lock).unwrap_or_default()
                );
            }

            Ok(response)
        })
    }
}
