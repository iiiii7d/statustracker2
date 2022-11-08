use maplit::hashmap;
use server::StatusTracker;
use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[rocket::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::registry()
        .with(fmt::layer().compact())
        .with(EnvFilter::from_env("RUST_LOG"))
        .init();
    server::start_server(
        StatusTracker::new(
            hashmap! {
                "Staff".into() => vec![
                    Uuid::new_v4()
                ]
            },
            "https://dynmap.minecartrapidtransit.net/standalone/dynmap_new.json"
                .parse()
                .unwrap(),
            std::env::var("MONGO").unwrap_or_default(),
            "new_mrt",
        )
        .await
        .unwrap(),
    )
    .await
    .unwrap();
}
