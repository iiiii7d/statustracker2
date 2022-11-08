use std::{sync::Arc, time::Duration};

use eyre::Result;
use futures::stream::StreamExt;
use mongodb::bson::doc;
use rocket::{
    fairing::{Fairing, Info, Kind},
    http::{Header, Status},
    response,
    response::{content, Responder},
    routes,
    serde::json::Json,
    Request, Response, State,
};
use serde::Serialize;
use tokio::sync::RwLock;
use tracing::{error, info};
use uuid::Uuid;

use crate::{hour::Hour, name_to_uuid::name_to_uuid, utils::HourTimestamp, StatusTracker};

#[derive(Debug)]
struct CustomMsgPack<T>(pub T);

impl<'r, T: Serialize> Responder<'r, 'static> for CustomMsgPack<T> {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
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

#[rocket::get("/?<from>&<to>")]
async fn range(
    tracker: &State<Arc<RwLock<StatusTracker>>>,
    from: HourTimestamp,
    to: HourTimestamp,
) -> Result<CustomMsgPack<Vec<Hour>>, String> {
    let a = tracker
        .inner()
        .read()
        .await
        .database
        .collection::<Hour>("hours")
        .find(
            doc! {
                "_id": {
                    "$gte": from,
                    "$lte": to
                }
            },
            None,
        )
        .await
        .map_err(|a| format!("Error while querying database: {a}"))?
        .collect::<Vec<_>>()
        .await;
    let a = a
        .into_iter()
        .collect::<mongodb::error::Result<Vec<_>>>()
        .map_err(|a| format!("Error while querying database: {a}"))?;
    Ok(CustomMsgPack(a))
}

#[rocket::get("/name_map")]
async fn name_map(tracker: &State<Arc<RwLock<StatusTracker>>>) -> CustomMsgPack<Vec<Uuid>> {
    info!("Retrieving name map");
    let a = &tracker.inner().read().await.name_map;
    CustomMsgPack(
        a.data
            .iter()
            .map(|bytes| Uuid::from_bytes(*bytes))
            .collect(),
    )
}

#[rocket::get("/uuid/<name>")]
async fn uuid_route(name: &str) -> Result<Json<Option<String>>, String> {
    Ok(Json(
        name_to_uuid(name)
            .await
            .map_err(|a| format!("Error while retrieving uuid: {a}"))?
            .map(|a| a.to_string()),
    ))
}

pub async fn start_server(tracker: StatusTracker) -> Result<()> {
    let tracker = Arc::new(RwLock::new(tracker));
    let r = rocket::build()
        .mount("/", routes![range, name_map, uuid_route])
        .attach(CORS)
        .manage(Arc::clone(&tracker))
        .ignite()
        .await?;

    let h = tokio::spawn(async move {
        loop {
            let mut tracker = tracker.write().await;
            let _ = tracker.run().await.map_err(|e| error!("{e}"));
            drop(tracker);
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    });
    let _ = r.launch().await?;
    h.abort();
    Ok(())
}
