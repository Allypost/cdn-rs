use std::path::PathBuf;

use args::ARGS;
use axum::{
    extract::Request,
    http::{header, HeaderValue},
    middleware::Next,
    Router,
};
use compression::Compression;
use once_cell::sync::OnceCell;
use strum::IntoEnumIterator;
use tokio::sync::mpsc::UnboundedSender;
use tower_http::{
    catch_panic::CatchPanicLayer,
    cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer},
    set_header::SetResponseHeaderLayer,
};
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod args;
mod compression;
mod tempfile;

static COMPRESS_SEND: OnceCell<UnboundedSender<(PathBuf, Compression)>> = OnceCell::new();

#[tokio::main]
async fn main() {
    init_logging();

    if !ARGS.serve_directory.is_dir() {
        error!(serve_directory = ?ARGS.serve_directory, "Serve directory does not exist or is not a directory.");
        std::process::exit(1);
    }

    if let Err(e) = std::fs::create_dir_all(ARGS.temp_dir()) {
        error!(
            dir = ?ARGS.temp_dir(),
            "Failed to create temp directory: {}", e
        );
        std::process::exit(1);
    }

    init_compress_worker();
    start_server().await;
}

fn init_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

fn init_compress_worker() {
    let (compress_send, mut compress_recv) =
        tokio::sync::mpsc::unbounded_channel::<(PathBuf, Compression)>();
    COMPRESS_SEND.get_or_init(|| compress_send);

    tokio::spawn(async move {
        while let Some((path, compression)) = compress_recv.recv().await {
            if let Err(e) = compression.compress_file(&path) {
                warn!("Failed to compress file {:?}: {}", &path, e);
            }
        }
    });
}

async fn start_server() {
    let listener = {
        let addr = format!("{}:{}", ARGS.host, ARGS.port);

        tokio::net::TcpListener::bind(&addr)
            .await
            .unwrap_or_else(|_| panic!("Failed to bind to {}", addr))
    };

    let app = Router::new()
        .nest_service(
            "/",
            tower_http::services::ServeDir::new(&ARGS.serve_directory)
                .precompressed_gzip()
                .precompressed_deflate()
                .precompressed_br()
                .precompressed_zstd()
                .append_index_html_on_directories(false),
        )
        .layer(SetResponseHeaderLayer::appending(
            header::VARY,
            HeaderValue::from_static("accept-encoding"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CACHE_CONTROL,
            HeaderValue::from_static(
                "public, max-age=3600, no-transform, stale-while-revalidate=600, stale-if-error=3600",
            ),
        ))
        .layer(axum::middleware::from_fn(
            |request: Request, next: Next| async move {
                if !ARGS.compress_files {
                    return next.run(request).await;
                }

                let file_path = {
                    let p = ARGS
                        .serve_directory
                        .join(format!("./{}", request.uri().path()));

                    std::fs::canonicalize(&p).unwrap_or(p)
                };

                let response = next.run(request).await;

                if response.status().is_success() {
                    if let Some(compress_send) = COMPRESS_SEND.get() {
                        for ct in Compression::iter() {
                            let _ = compress_send.send((file_path.clone(), ct));
                        }
                    }
                }

                response
            },
        ))
        .layer(
            CorsLayer::new()
                .allow_methods(AllowMethods::mirror_request())
                .allow_origin(AllowOrigin::mirror_request())
                .allow_headers(AllowHeaders::mirror_request()),
        )
        .layer(CatchPanicLayer::new());

    info!(
        "Listening on http://{}",
        listener.local_addr().expect("Failed to get local address")
    );

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
