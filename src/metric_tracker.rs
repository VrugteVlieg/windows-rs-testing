use std::sync::{Arc, Mutex};

use crate::{roaming::UxiRoamEvent, windows_api_client::WindowsApiClient};

use state::InitCell;

static GLOBAL_METRIC_TRACKER: InitCell<MetricTracker> = InitCell::new();

pub struct MetricTracker {
    roam_events: Arc<Mutex<Vec<UxiRoamEvent>>>,
}

impl MetricTracker {
    pub fn init() {
        tokio::spawn(async {
            let mut signal_lvl: u32 = 0;
            let mut rx = WindowsApiClient::track_signal_changes();
            loop {
                let val = rx.recv().await.unwrap();
                println!("Signal quality change: {} -> {}", signal_lvl, val);
                signal_lvl = val;
            }
        });

        tokio::spawn(async {
            let mut rx = WindowsApiClient::track_roaming_events();
            loop {
                match rx.recv().await {
                    Ok(event) => {
                        println!("Received uxi roam event {event:?}");
                        (*(GLOBAL_METRIC_TRACKER.get().roam_events.lock().unwrap())).push(event);
                    }
                    Err(e) => {
                        println!("Error receiving uxi roam event {e:?}");
                    }
                }
            }
        });

        GLOBAL_METRIC_TRACKER.set(MetricTracker {
            roam_events: Arc::new(Mutex::new(vec![])),
        });
    }

    pub fn get_roam_events() -> Vec<UxiRoamEvent> {
        (*(GLOBAL_METRIC_TRACKER.get().roam_events.lock().unwrap()))
            .drain(0..)
            .collect()
    }
}
