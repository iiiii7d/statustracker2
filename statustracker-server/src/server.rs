use std::{sync::Arc, time::Duration};

use color_eyre::eyre::Result;
use mongodb::bson::doc;
use rocket::{
    fairing::{Fairing, Info, Kind},
    http::{uri::Host, Header, Status},
    response,
    response::{content, status::BadRequest, Redirect, Responder},
    routes, Request, Response, State,
};
use serde::Serialize;
use tokio::{sync::RwLock, time::Instant};
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    hour::RollingAvgRecord, name_to_uuid::name_to_uuid, tracker::StatusTracker,
    utils::MinuteTimestamp,
};

#[derive(Debug)]
struct CustomMsgPack<T>(pub T);

impl<'r, T: Serialize> Responder<'r, 'static> for CustomMsgPack<T> {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        #[allow(clippy::map_err_ignore)]
        let buf = rmp_serde::to_vec_named(&self.0).map_err(|_| Status::InternalServerError)?;

        content::RawMsgPack(buf).respond_to(req)
    }
}

// https://stackoverflow.com/questions/62412361/how-to-set-up-cors-or-options-for-rocket-rs
pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[rocket::get("/?<from>&<to>&<range>")]
async fn range(
    tracker: &State<Arc<RwLock<StatusTracker>>>,
    from: MinuteTimestamp,
    to: MinuteTimestamp,
    range: u64,
) -> Result<CustomMsgPack<Vec<Option<RollingAvgRecord>>>, String> {
    if to - from > 60 * 24 * 365 * 5 {
        return Err("Duration is too long".into());
    };
    let a = tracker
        .read()
        .await
        .database
        .get_rolling_avg(from, to, range)
        .await
        .map_err(|a| format!("Error reading from database: {a}"))?;
    Ok(CustomMsgPack(a))
}

#[rocket::get("/name_map")]
async fn name_map(tracker: &State<Arc<RwLock<StatusTracker>>>) -> CustomMsgPack<Vec<String>> {
    info!("Retrieving name map");
    let a = &tracker.read().await.name_map;
    CustomMsgPack(
        a.data
            .iter()
            .map(|bytes| Uuid::from_bytes(*bytes).to_string())
            .collect(),
    )
}

#[rocket::get("/uuid/<name>")]
async fn uuid_route(name: &str) -> Result<CustomMsgPack<Option<String>>, String> {
    Ok(CustomMsgPack(
        name_to_uuid(name)
            .await
            .map_err(|a| format!("Error while retrieving uuid: {a}"))?
            .map(|a| a.to_string()),
    ))
}

#[rocket::get("/")]
async fn redirect_to_client(
    tracker: &State<Arc<RwLock<StatusTracker>>>,
    host: &Host<'_>,
) -> Redirect {
    info!(%host, "Redirecting to client");
    let hosted_over_http = tracker.read().await.config.hosted_over_http;
    Redirect::to(format!(
        "https://iiiii7d.github.io/statustracker2/?server=http{}://{}",
        if hosted_over_http { "" } else { "s" },
        host
    ))
}

pub async fn start_server(tracker: StatusTracker) -> Result<()> {
    let tracker = Arc::new(RwLock::new(tracker));
    let r = rocket::build()
        .mount(
            "/",
            routes![range, name_map, uuid_route, redirect_to_client],
        )
        .attach(CORS)
        .manage(Arc::clone(&tracker))
        .ignite()
        .await?;

    let h = tokio::spawn(async move {
        loop {
            let start = Instant::now();
            let mut tracker = tracker.write().await;
            let _ = tracker.run().await.map_err(|e| error!("{e}"));
            drop(tracker);
            let time_taken = Instant::now() - start;
            info!(?time_taken);
            tokio::time::sleep(Duration::from_secs(60) - time_taken).await;
        }
    });
    let _ = r.launch().await?;
    h.abort();
    Ok(())
}
