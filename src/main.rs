use axum::{
    extract::Extension,
    response::Json,
    routing::get,
    Router
};
use btleplug::api::{ Central, Manager as _, Peripheral as _, ScanFilter };
use btleplug::platform::Manager;
use serde::Serialize;
use tokio::time::interval;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

#[derive(Serialize, Clone)]
struct Device {
    name: Option<String>,
    address: String
}

struct AppState {
    devices: Arc<RwLock<Vec<Device>>>,
}

#[tokio::main]
async fn main(){
    let manager = Manager::new().await.unwrap();
    let adapters = manager.adapters().await.unwrap();
    let adapter = adapters.into_iter().next().expect("Nenhum adaptador bluetooth encontrado!");

    let state = Arc::new(AppState {
        devices: Arc::new(RwLock::new(vec![])),
    });

    let state_clone = state.clone();

    tokio::spawn(async move {
        let _ = adapter.start_scan(ScanFilter::default()).await.unwrap();
        println!("Procurando por dispositivos...");

        let mut interval = interval(Duration::from_secs(2));
        loop {
            interval.tick().await;

            if let Ok(peripherals) = adapter.peripherals().await {
                let mut devices = state_clone.devices.write().await;
                devices.clear();

                for peripheral in peripherals {
                    if let Some(properties) = peripheral.properties().await.unwrap() {
                        let device = Device {
                            name: properties.local_name,
                            address: properties.address.to_string()
                        };

                        devices.push(device);
                    }
                }
            }
        }
    });

    let app = Router::new()
        .route("/devices", get(get_devices))
        .layer(Extension(state));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app.into_make_service()).await.unwrap();
}

async fn get_devices(Extension(state): Extension<Arc<AppState>>) -> Json<Vec<Device>> {
    let devices = state.devices.read().await;
    Json(devices.clone())
}