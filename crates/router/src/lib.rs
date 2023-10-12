#![forbid(unsafe_code)]
#![recursion_limit = "256"]

#[cfg(feature = "stripe")]
pub mod compatibility;
pub mod configs;
pub mod connection;
pub mod connector;
pub(crate) mod consts;
pub mod core;
pub mod cors;
pub mod db;
pub mod env;
pub(crate) mod macros;
pub mod routes;
pub mod workflows;

pub mod middleware;
pub mod openapi;
pub mod services;
pub mod types;
pub mod utils;

use std::io::Write;

use actix_web::{
    body::MessageBody,
    dev::{Server, ServerHandle, ServiceFactory, ServiceRequest},
    middleware::ErrorHandlers,
};
use http::StatusCode;
use pprof::protos::Message;
use routes::AppState;
use storage_impl::errors::ApplicationResult;
use tokio::sync::{mpsc, oneshot};

pub use self::env::logger;
use crate::{
    configs::settings,
    core::errors::{self},
};

#[cfg(feature = "mimalloc")]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// Header Constants
pub mod headers {
    pub const ACCEPT: &str = "Accept";
    pub const API_KEY: &str = "API-KEY";
    pub const APIKEY: &str = "apikey";
    pub const X_CC_API_KEY: &str = "X-CC-Api-Key";
    pub const AUTHORIZATION: &str = "Authorization";
    pub const CONTENT_TYPE: &str = "Content-Type";
    pub const DATE: &str = "Date";
    pub const NONCE: &str = "nonce";
    pub const TIMESTAMP: &str = "Timestamp";
    pub const TOKEN: &str = "token";
    pub const X_API_KEY: &str = "X-API-KEY";
    pub const X_API_VERSION: &str = "X-ApiVersion";
    pub const X_FORWARDED_FOR: &str = "X-Forwarded-For";
    pub const X_MERCHANT_ID: &str = "X-Merchant-Id";
    pub const X_LOGIN: &str = "X-Login";
    pub const X_TRANS_KEY: &str = "X-Trans-Key";
    pub const X_VERSION: &str = "X-Version";
    pub const X_CC_VERSION: &str = "X-CC-Version";
    pub const X_ACCEPT_VERSION: &str = "X-Accept-Version";
    pub const X_DATE: &str = "X-Date";
    pub const X_WEBHOOK_SIGNATURE: &str = "X-Webhook-Signature-512";
    pub const X_REQUEST_ID: &str = "X-Request-Id";
    pub const STRIPE_COMPATIBLE_WEBHOOK_SIGNATURE: &str = "Stripe-Signature";
}

pub mod pii {
    //! Personal Identifiable Information protection.

    pub(crate) use common_utils::pii::Email;
    #[doc(inline)]
    pub use masking::*;
}

use actix_web_opentelemetry::{RequestMetrics, RequestTracing};
use opentelemetry::{self, global};
use router_env::{
    tracing,
    tracing_subscriber::{self, prelude::*},
};

#[derive(Debug, Clone)]
pub struct OpenTelemetryStack {
    request_metrics: RequestMetrics,
}

impl Default for OpenTelemetryStack {
    fn default() -> Self {
        let app_name = std::env::var("CARGO_BIN_NAME").unwrap_or("demo".to_string());

        global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());

        #[allow(clippy::expect_used)]
        let tracer = opentelemetry_jaeger::new_agent_pipeline()
            .with_endpoint(std::env::var("JAEGER_ENDPOINT").unwrap_or("localhost:6831".to_string()))
            .with_service_name(app_name.clone())
            //.with_auto_split_batch(true)
            .install_batch(opentelemetry::runtime::Tokio)
            .expect("Failed to install OpenTelemetry tracer.");

        let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

        #[allow(clippy::expect_used)]
        let subscriber = tracing_subscriber::Registry::default().with(telemetry);
        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to install `tracing` subscriber.");

        let request_metrics = RequestMetrics::default();
        Self { request_metrics }
    }
}

impl OpenTelemetryStack {
    pub fn metrics(&self) -> RequestMetrics {
        self.request_metrics.clone()
    }
}

pub fn mk_app(
    state: AppState,
    request_body_limit: usize,
) -> actix_web::App<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse<impl MessageBody>,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    let mut server_app = get_application_builder(request_body_limit);

    #[cfg(feature = "openapi")]
    {
        use utoipa::OpenApi;
        server_app = server_app.service(
            utoipa_swagger_ui::SwaggerUi::new("/docs/{_:.*}")
                .url("/docs/openapi.json", openapi::ApiDoc::openapi()),
        );
    }

    #[cfg(feature = "dummy_connector")]
    {
        use routes::DummyConnector;
        server_app = server_app.service(DummyConnector::server(state.clone()));
    }

    #[cfg(any(feature = "olap", feature = "oltp"))]
    {
        #[cfg(feature = "olap")]
        {
            // This is a more specific route as compared to `MerchantConnectorAccount`
            // so it is registered before `MerchantConnectorAccount`.
            server_app = server_app.service(routes::BusinessProfile::server(state.clone()))
        }
        server_app = server_app
            .service(routes::Payments::server(state.clone()))
            .service(routes::Customers::server(state.clone()))
            .service(routes::Configs::server(state.clone()))
            .service(routes::Refunds::server(state.clone()))
            .service(routes::MerchantConnectorAccount::server(state.clone()))
            .service(routes::Mandates::server(state.clone()))
    }

    #[cfg(feature = "oltp")]
    {
        server_app = server_app
            .service(routes::PaymentMethods::server(state.clone()))
            .service(routes::EphemeralKey::server(state.clone()))
            .service(routes::Webhooks::server(state.clone()))
            .service(routes::PaymentLink::server(state.clone()));
    }

    #[cfg(feature = "olap")]
    {
        server_app = server_app
            .service(routes::MerchantAccount::server(state.clone()))
            .service(routes::ApiKeys::server(state.clone()))
            .service(routes::Files::server(state.clone()))
            .service(routes::Disputes::server(state.clone()))
    }

    #[cfg(all(feature = "olap", feature = "kms"))]
    {
        server_app = server_app.service(routes::Verify::server(state.clone()));
    }

    #[cfg(feature = "payouts")]
    {
        server_app = server_app.service(routes::Payouts::server(state.clone()));
    }

    #[cfg(feature = "stripe")]
    {
        server_app = server_app.service(routes::StripeApis::server(state.clone()));
    }
    server_app = server_app.service(routes::Cards::server(state.clone()));
    server_app = server_app.service(routes::Cache::server(state.clone()));
    server_app = server_app.service(routes::Health::server(state));

    server_app
}

/// Starts the server
///
/// # Panics
///
///  Unwrap used because without the value we can't start the server
#[allow(clippy::expect_used, clippy::unwrap_used)]
pub async fn start_server(conf: settings::Settings) -> ApplicationResult<Server> {
    logger::debug!(startup_config=?conf);
    let server = conf.server.clone();
    let (tx, rx) = oneshot::channel();
    let api_client = Box::new(
        services::ProxyClient::new(
            conf.proxy.clone(),
            services::proxy_bypass_urls(&conf.locker),
        )
        .map_err(|error| {
            errors::ApplicationError::ApiClientError(error.current_context().clone())
        })?,
    );
    let state = routes::AppState::new(conf, tx, api_client).await;
    let request_body_limit = server.request_body_limit;
    let server = actix_web::HttpServer::new(move || mk_app(state.clone(), request_body_limit))
        .bind((server.host.as_str(), server.port))?
        .workers(server.workers)
        .shutdown_timeout(server.shutdown_timeout)
        .run();
    tokio::spawn(receiver_for_error(rx, server.handle()));
    Ok(server)
}

pub async fn receiver_for_error(rx: oneshot::Receiver<()>, mut server: impl Stop) {
    match rx.await {
        Ok(_) => {
            logger::error!("The redis server failed ");
            server.stop_server().await;
        }
        Err(err) => {
            logger::error!("Channel receiver error{err}");
        }
    }
}

#[async_trait::async_trait]
pub trait Stop {
    async fn stop_server(&mut self);
}

#[async_trait::async_trait]
impl Stop for ServerHandle {
    async fn stop_server(&mut self) {
        let _ = self.stop(true).await;
    }
}
#[async_trait::async_trait]
impl Stop for mpsc::Sender<()> {
    async fn stop_server(&mut self) {
        let _ = self.send(()).await.map_err(|err| logger::error!("{err}"));
    }
}

pub fn get_application_builder(
    request_body_limit: usize,
) -> actix_web::App<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse<impl MessageBody>,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    let json_cfg = actix_web::web::JsonConfig::default()
        .limit(request_body_limit)
        .content_type_required(true)
        .error_handler(utils::error_parser::custom_json_error_handler);

    let _prof_guard = pprof::ProfilerGuardBuilder::default()
        .frequency(1000)
        .blocklist(&["libc", "libgcc", "pthread", "vdso"])
        .build()
        .unwrap();

    let now = common_utils::date_time::now();

    let file_path = std::env::var("FILE_PATH").unwrap_or("/mnt/reports".to_string());

    if let Ok(report) = _prof_guard.report().build() {
        let file = std::fs::File::create(format!("{file_path}/flamegraph_{now}.svg")).unwrap();
        let mut options = pprof::flamegraph::Options::default();
        options.image_width = Some(2500);
        report.flamegraph_with_options(file, &mut options).unwrap();
    };

    if let Ok(report) = _prof_guard.report().build() {
        let mut file = std::fs::File::create(format!("{file_path}/profile_{now}.pb")).unwrap();
        let profile = report.pprof().unwrap();

        let mut content = Vec::new();
        profile.write_to_vec(&mut content).unwrap();
        file.write_all(&content).unwrap();
    };

    println!("Report generated");

    actix_web::App::new()
        .app_data(json_cfg)
        .wrap(ErrorHandlers::new().handler(
            StatusCode::NOT_FOUND,
            errors::error_handlers::custom_error_handlers,
        ))
        .wrap(ErrorHandlers::new().handler(
            StatusCode::METHOD_NOT_ALLOWED,
            errors::error_handlers::custom_error_handlers,
        ))
        .wrap(middleware::default_response_headers())
        .wrap(middleware::RequestId)
        .wrap(cors::cors())
        //.wrap(router_env::tracing_actix_web::TracingLogger::default())
        .wrap(RequestTracing::new())
        .wrap(OpenTelemetryStack::default().metrics())
}
