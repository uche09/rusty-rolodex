use rusty_rolodex::prelude::{AppError, run_app};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    run_app().await
}
