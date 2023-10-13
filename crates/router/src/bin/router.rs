use pprof::protos::Message;
use router::{
    configs::settings::{CmdLineConf, Settings},
    core::errors::{ApplicationError, ApplicationResult},
    logger,
};
use std::io::Write;

#[actix_web::main]
async fn main() -> ApplicationResult<()> {
    // get commandline config before initializing config
    let cmd_line = <CmdLineConf as clap::Parser>::parse();

    #[cfg(feature = "openapi")]
    {
        use router::configs::settings::Subcommand;
        if let Some(Subcommand::GenerateOpenapiSpec) = cmd_line.subcommand {
            let file_path = "openapi/openapi_spec.json";
            #[allow(clippy::expect_used)]
            std::fs::write(
                file_path,
                <router::openapi::ApiDoc as utoipa::OpenApi>::openapi()
                    .to_pretty_json()
                    .expect("Failed to serialize OpenAPI specification as JSON"),
            )
            .expect("Failed to write OpenAPI specification to file");
            println!("Successfully saved OpenAPI specification file at '{file_path}'");
            return Ok(());
        }
    }

    #[allow(clippy::expect_used)]
    let conf = Settings::with_config_path(cmd_line.config_path)
        .expect("Unable to construct application configuration");
    #[allow(clippy::expect_used)]
    conf.validate()
        .expect("Failed to validate router configuration");

    let _guard = router_env::setup(
        &conf.log,
        router_env::service_name!(),
        [router_env::service_name!(), "actix_server"],
    );

    logger::info!("Application started [{:?}] [{:?}]", conf.server, conf.log);

    let _prof_guard = pprof::ProfilerGuardBuilder::default()
        .frequency(1000)
        .blocklist(&["libc", "libgcc", "pthread", "vdso"])
        .build()
        .unwrap();

    let now = common_utils::date_time::now();

    let file_path = std::env::var("FILE_PATH").unwrap_or("/mnt/reports".to_string());

    #[allow(clippy::expect_used)]
    let server = router::start_server(conf)
        .await
        .expect("Failed to create the server");
    let _ = server.await;

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

    Err(ApplicationError::from(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Server shut down",
    )))
}
