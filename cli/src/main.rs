mod cli_component;

use cli_component::{AppError, run_app};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    run_app().await
}
