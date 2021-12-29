

#[macro_use] extern crate rocket;

use std::sync::{Arc};
use futures_util::lock::Mutex;
use futures_util::join;
use instance::ServerInstance;
use instance_manager::InstanceManager;
use rocket::{response::content};
use rocket::{State};
use serde_json::{Value};
use std::sync::atomic::{AtomicUsize};
use std::path::Path;
use chashmap::CHashMap;
mod instance;
mod util;
mod handlers;
mod instance_manager;
use handlers::jar;
use mongodb::{bson::doc, options::ClientOptions, sync::Client};

struct HitCount {
    count: AtomicUsize
}

pub struct MyManagedState {
    instance_manager : Arc<Mutex<InstanceManager>>,
    download_status: CHashMap<String, (u64, u64)>,
    mongoDBClient: Client
}

#[get("/api/new/<instance_name>/<version>")]
async fn setup(instance_name : String, version : String, state: &State<MyManagedState>) -> String {
    let mut manager = state.instance_manager.lock().await;
    manager.create_instance(instance_name, version, None, state).await.unwrap()
}

#[get("/api/status/<instance_name>")]
async fn download_status(instance_name : String, state: &State<MyManagedState>) -> String {
    if !state.download_status.contains_key(&instance_name) {
        return "does not exists".to_string();
    }
    return format!("{}/{}", state.download_status.get(&instance_name).unwrap().0, state.download_status.get(&instance_name).unwrap().1)
}


// #[get("/count")]
// async fn test(hit_count: &State<HitCount>) -> String {
//     let current_count = hit_count.count.load(Ordering::Relaxed);
//     hit_count.count.store(current_count + 1, Ordering::Relaxed);
//     format!("Number of visits: {}", current_count)
// }

#[get("/api/start/<uuid>")]
async fn start(state: &State<MyManagedState>, uuid : String) -> String {
    state.instance_manager.lock().await.start_instance(uuid).unwrap();
    "Ok".to_string()
}

#[get("/api/stop/<uuid>")]
async fn stop(state: &State<MyManagedState>, uuid : String) -> String {
    state.instance_manager.lock().await.stop_instance(uuid).unwrap();
    "Ok".to_string()
}

#[get("/api/send/<uuid>/<command>")]
async fn send(uuid : String, command: String, state: &State<MyManagedState>) -> String {
    state.instance_manager.lock().await.send_command(uuid, command).unwrap();
    "Ok".to_string()
}

#[launch]
async fn rocket() -> _ {

    let mut client_options = ClientOptions::parse("mongodb://localhost:27017").unwrap();
    client_options.app_name = Some("MongoDB Client".to_string());

    let client = Client::with_options(client_options).unwrap();

    rocket::build()
    .mount("/", routes![start, stop, send, setup, download_status, jar::get_vanilla_versions, jar::get_vanilla_jar])
    .manage(MyManagedState{
        instance_manager : Arc::new(Mutex::new(InstanceManager::new( "/home/peter/Lodestone/backend/InstanceTest/".to_string(), client.clone()))),
        download_status: CHashMap::new(),
        mongoDBClient: client
    })
}