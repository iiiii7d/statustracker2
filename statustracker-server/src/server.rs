use std::{io::Cursor, sync::Arc, time::Duration};

use color_eyre::{
    eyre::{eyre, Result},
    Report,
};
use mongodb::bson::doc;
use rocket::{
    fairing::{Fairing, Info, Kind},
    http::{uri::Host, Header, Status},
    response,
    response::{content, Redirect, Responder},
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
        let buf = match rmp_serde::to_vec_named(&self.0) {
            Ok(b) => b,
            Err(e) => return CustomError::from(Report::from(e)).respond_to(req),
        };

        content::RawMsgPack(buf).respond_to(req)
    }
}
#[derive(Debug)]
struct CustomError(pub Status, pub Report);

impl<'r> Responder<'r, 'static> for CustomError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        Response::build()
            .status(self.0)
            .streamed_body(Cursor::new(self.1.to_string()))
            .ok()
    }
}
impl From<Report> for CustomError {
    fn from(value: Report) -> Self {
        Self(Status::InternalServerError, value)
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
) -> Result<CustomMsgPack<Vec<Option<RollingAvgRecord>>>, CustomError> {
    if to - from > 60 * 24 * 365 * 5 {
        return Err(CustomError(
            Status::BadRequest,
            eyre!("Duration is too long"),
        ));
    };
    let a = tracker
        .read()
        .await
        .database
        .get_rolling_avg(from, to, range)
        .await?;
    Ok(CustomMsgPack(a))
}

#[rocket::get("/player/<name>?<from>&<to>")]
async fn player(
    tracker: &State<Arc<RwLock<StatusTracker>>>,
    name: &str,
    from: MinuteTimestamp,
    to: MinuteTimestamp,
) -> Result<CustomMsgPack<Vec<(MinuteTimestamp, MinuteTimestamp)>>, CustomError> {
    if to - from > 60 * 24 * 365 * 5 {
        return Err(CustomError(
            Status::BadRequest,
            eyre!("Duration is too long"),
        ));
    };
    let tracker = tracker.read().await;
    let uuid = name_to_uuid(name).await?.unwrap_or_default();
    let Some((i, _)) = tracker
        .name_map
        .data
        .iter()
        .enumerate()
        .find(|(_, a)| *a == uuid.as_bytes())
    else {
        return Ok(CustomMsgPack(Vec::new()));
    };
    let a = tracker.database.get_player_join_times(from, to, i).await?;

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
async fn uuid_route(name: &str) -> Result<CustomMsgPack<Option<String>>, CustomError> {
    Ok(CustomMsgPack(
        name_to_uuid(name).await?.map(|a| a.to_string()),
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
    let no_write = tracker.config.no_write;
    let tracker = Arc::new(RwLock::new(tracker));
    let r = rocket::build()
        .mount(
            "/",
            routes![range, name_map, player, uuid_route, redirect_to_client],
        )
        .attach(CORS)
        .manage(Arc::clone(&tracker))
        .ignite()
        .await?;

    let h = (!no_write).then(|| {
        tokio::spawn(async move {
            loop {
                let start = Instant::now();
                let mut tracker = tracker.write().await;
                let _ = tracker.run().await.map_err(|e| error!("{e}"));
                drop(tracker);
                let time_taken = Instant::now() - start;
                info!(?time_taken);
                tokio::time::sleep(Duration::from_secs(60) - time_taken).await;
            }
        })
    });

    let _ = r.launch().await?;
    if let Some(h) = h {
        h.abort()
    }
    Ok(())
}
