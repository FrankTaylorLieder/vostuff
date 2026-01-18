use axum::{Router, routing::post};
use leptos::*;
use leptos_axum::{LeptosRoutes, generate_route_list};
use std::env;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Get API base URL from environment
    let api_base_url =
        env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

    tracing::info!("API base URL: {}", api_base_url);

    // Get Leptos configuration
    // cargo-leptos sets LEPTOS_OUTPUT_NAME when running
    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(vostuff_web::App);

    tracing::info!("Leptos routes generated: {} routes", routes.len());

    // Build the Axum router
    let app = Router::new()
        // Serve static files from the public directory
        .nest_service("/pkg", ServeDir::new("./target/site/pkg"))
        .nest_service("/style", ServeDir::new("./crates/vostuff-web/style"))
        .route("/api/*fn", post(leptos_axum::handle_server_fns))
        .leptos_routes(&leptos_options, routes, || view! { <vostuff_web::App/> })
        .with_state(leptos_options);

    tracing::info!("VOStuff Web Server starting on {}", addr);
    tracing::info!("Visit http://{}", addr);

    // Start the server
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
