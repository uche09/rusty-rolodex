use api::{config, error::ApiError, routes::create_router, state};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};



#[tokio::main]
async fn main() -> Result<(), ApiError> {
    // Logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "my_api=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load config
    let config = config::Config::from_env()?;

    // Build State
    let state = state::ApiState::new(config.clone()).await?;

    // Build router
    let app = create_router(state);

    // Bind to host port and serve
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind listener");

    println!("Server running on http://localhost:3000");

    axum::serve(listener, app)
        .await
        .expect("Filed to start server");

    Ok(())
}