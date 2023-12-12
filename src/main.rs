pub mod utils;
pub mod windows_api_client;
pub mod windows_type_wrappers;

use metric_tracker::MetricTracker;
use utils::NetworkBand;
use windows::Win32::NetworkManagement::WiFi::{WLAN_AVAILABLE_NETWORK, WLAN_BSS_ENTRY};
use windows_api_client::WindowsApiClient;
pub mod metric_tracker;
pub mod roaming;
pub mod roaming_windows;


#[tokio::main]
async fn main() {

        WindowsApiClient::init();
        MetricTracker::init();

        // let target_ssid = utils::create_dot_11_ssid("Hello World Too");
        let mut counter = 0;

        loop {
            println!("Starting cycle {counter} @ {:?}", std::time::SystemTime::now());
            counter+=1;


            std::thread::sleep(std::time::Duration::from_secs(20));
            let roam_events = MetricTracker::get_roam_events();
            println!("Roam events in last cycle:\n{roam_events:#?}");
        }
}



#[derive(Debug)]
pub struct Network {
    pub ssid: String,
    pub bssid: String,
    pub rssi: i32,
    pub channel: u32,
    pub band: String,
    pub secured: bool
}

impl From<(&WLAN_BSS_ENTRY, &WLAN_AVAILABLE_NETWORK)> for Network {
    fn from((bss_info, network_info): (&WLAN_BSS_ENTRY, &WLAN_AVAILABLE_NETWORK)) -> Self {
        let ssid = utils::parse_ssid(network_info.dot11Ssid);
        let bssid =  utils::parse_bssid(bss_info.dot11Bssid);
        let rssi = bss_info.lRssi;
        let channel = utils::map_freq_to_channel(bss_info.ulChCenterFrequency);
        let band = NetworkBand::try_from(bss_info.ulChCenterFrequency).unwrap().to_string();

        //https://learn.microsoft.com/en-us/windows/win32/nativewifi/dot11-auth-algorithm
        let secured = 1i32 != network_info.dot11DefaultAuthAlgorithm.0;

        Network { ssid, bssid, rssi, channel, band, secured }
    }
} 

impl std::fmt::Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} @ {}", self.ssid, self.bssid)
    }
}



// Success Roaming
//15:37:39 Windows notfication MSM::Roam start:
// Lyco HQ_5G @ 6E:12:B6:89:3A:0E
// profile: Lyco HQ_5G
// reason: 0
// 15:37:43 Windows notfication MSM::Authenticating:
// Lyco HQ_5G @ B4:0F:3B:BB:82:24
// profile: Lyco HQ_5G
// reason: 0
// Roaming state transition AttemptingRoam -> Authenticating(0)
// UnqualifiedSuccess is terminal
// Sending Roam(NoErrors)
// 15:37:43 Windows notfication MSM::Roam end:
// Lyco HQ_5G @ B4:0F:3B:BB:82:24
// profile: Lyco HQ_5G
// reason: 0
// Received uxi roam event Roam(NoErrors)



//Roaming failed -> fallback network
// Windows notfication Roam start:
// Lyco HQ_5G @ 6E:12:B6:89:3A:0E
// profile: Lyco HQ_5G
// result: 0
// Windows notfication Authenticating:
// Lyco HQ_5G @ B4:0F:3B:BB:82:24
// profile: Lyco HQ_5G
// result: 0
// Unable to parse notifcation data L2_NOTIFICATION_DATA { NotificationSource: 16, NotificationCode: 59, InterfaceGuid: A33653CA-6496-4031-A115-3F02DBDDC487, dwDataSize: 16, pData: 0x2c92ae05bd0 }
// Windows notfication Msm(SignalQualityChange)
// Windows notfication Acm(ScanComplete)
// Windows notfication Msm(SignalQualityChange)
// Windows notfication Roam start:
// Lyco HQ_5G @ B4:0F:3B:BB:82:24
// profile: Lyco HQ_5G
// result: 0
// Windows notfication Authenticating:
// Lyco HQ_5G @ B4:0F:3B:BB:82:24
// profile: Lyco HQ_5G
// result: 0
// Unable to parse notifcation data L2_NOTIFICATION_DATA { NotificationSource: 16, NotificationCode: 59, InterfaceGuid: A33653CA-6496-4031-A115-3F02DBDDC487, dwDataSize: 16, pData: 0x2c92ae05a50 }
// Windows notfication Msm(SignalQualityChange)
// Windows notfication Msm(SignalQualityChange)
// Windows notfication Roam start:
// Lyco HQ_5G @ B4:0F:3B:BB:82:24
// profile: Lyco HQ_5G
// result: 0
// Windows notfication Authenticating:
// Lyco HQ_5G @ B4:0F:3B:BB:82:24
// profile: Lyco HQ_5G
// result: 0
// Unable to parse notifcation data L2_NOTIFICATION_DATA { NotificationSource: 16, NotificationCode: 59, InterfaceGuid: A33653CA-6496-4031-A115-3F02DBDDC487, dwDataSize: 16, pData: 0x2c92ae05e40 }
// Windows notfication Msm(SignalQualityChange)
// Windows notfication Msm(SignalQualityChange)
// Windows notfication Roam start:
// Lyco HQ_5G @ B4:0F:3B:BB:82:24
// profile: Lyco HQ_5G
// result: 0
// Windows notfication Authenticating:
// Lyco HQ_5G @ B4:0F:3B:BB:82:24
// profile: Lyco HQ_5G
// result: 0
// Unable to parse notifcation data L2_NOTIFICATION_DATA { NotificationSource: 16, NotificationCode: 59, InterfaceGuid: A33653CA-6496-4031-A115-3F02DBDDC487, dwDataSize: 16, pData: 0x2c92ae05a20 }
// Windows notfication Msm(SignalQualityChange)
// Windows notfication Msm(SignalQualityChange)
// Windows notfication Roam start:
// Lyco HQ_5G @ B4:0F:3B:BB:82:24
// profile: Lyco HQ_5G
// result: 0
// Windows notfication Authenticating:
// Lyco HQ_5G @ B4:0F:3B:BB:82:24
// profile: Lyco HQ_5G
// result: 0
// Unable to parse notifcation data L2_NOTIFICATION_DATA { NotificationSource: 16, NotificationCode: 59, InterfaceGuid: A33653CA-6496-4031-A115-3F02DBDDC487, dwDataSize: 16, pData: 0x2c92ae05ea0 }
// Windows notfication Msm(SignalQualityChange)
// Windows notfication Msm(SignalQualityChange)
// Windows notfication Roam start:
// Lyco HQ_5G @ B4:0F:3B:BB:82:24
// profile: Lyco HQ_5G
// result: 0
// Windows notfication Msm(Disconnected)
// Windows notfication Acm(Disconnected)
// Windows notfication Acm(ScanComplete)
// Windows notfication Acm(ScanListRefresh)
// Windows notfication Acm(ConnectionStart)
// Windows notfication Msm(Associating)
// Unable to parse notifcation data L2_NOTIFICATION_DATA { NotificationSource: 16, NotificationCode: 59, InterfaceGuid: A33653CA-6496-4031-A115-3F02DBDDC487, dwDataSize: 16, pData: 0x2c92ae05810 }
// Windows notfication Msm(Associated)
// Windows notfication Authenticating:
// Lyco HQ @ B4:0F:3B:BB:82:14
// profile: Lyco HQ
// result: 0




// Reconnect failed
// Received uxi roam event Roam(Disconnection(["Roam failed after 5 retries"]))
// 15:31:18 Windows notfication ACM::Disconnected:
// Lyco HQ_5G
// reason: 0
// 15:31:21 Windows notfication Acm(ScanComplete)
// 15:31:21 Windows notfication Acm(ScanListRefresh)
// 15:31:21 Windows notfication Acm(ProfilesExhausted)
// 15:31:21 Windows notfication Acm(NetworkAvailable)
// 15:31:22 Windows notfication ACM::ConnectionStart:
// Lyco HQ_5G
// reason: 0
// 15:31:22 Windows notfication MSM::Associating:
// Lyco HQ_5G @ 00:00:00:00:00:00
// profile: Lyco HQ_5G
// reason: 0
// 15:31:22 Windows notfication MSM::Associated:
// Lyco HQ_5G @ B4:0F:3B:BB:82:24
// profile: Lyco HQ_5G
// reason: 0
// 15:31:22 Windows notfication MSM::Authenticating:
// Lyco HQ_5G @ B4:0F:3B:BB:82:24
// profile: Lyco HQ_5G
// reason: 0
// 15:31:26 Windows notfication MSM::Authenticating:
// Lyco HQ_5G @ B4:0F:3B:BB:82:24
// profile: Lyco HQ_5G
// reason: 0
// 15:31:30 Windows notfication MSM::Authenticating:
// Lyco HQ_5G @ B4:0F:3B:BB:82:24
// profile: Lyco HQ_5G
// reason: 0
// 15:31:34 Windows notfication MSM::Authenticating:
// Lyco HQ_5G @ B4:0F:3B:BB:82:24
// profile: Lyco HQ_5G
// reason: 0
// 15:31:38 Windows notfication MSM::Authenticating:
// Lyco HQ_5G @ B4:0F:3B:BB:82:24
// profile: Lyco HQ_5G
// reason: 0
// 15:31:43 Windows notfication ACM::ConnectionComplete:
// Lyco HQ_5G
// reason: 294932



// Reconnect success
// 15:34:52 Windows notfication ACM::ConnectionStart:
// Lyco HQ_5G
// reason: 0
// 15:34:52 Windows notfication MSM::Associating:
// Lyco HQ_5G @ 00:00:00:00:00:00
// profile: Lyco HQ_5G
// reason: 0
// 15:34:52 Windows notfication MSM::Associated:
// Lyco HQ_5G @ 6E:12:B6:89:3A:0E
// profile: Lyco HQ_5G
// reason: 0
// 15:34:52 Windows notfication MSM::Authenticating:
// Lyco HQ_5G @ 6E:12:B6:89:3A:0E
// profile: Lyco HQ_5G
// reason: 0
// 15:34:52 Windows notfication Msm(Connected)
// 15:34:53 Windows notfication ACM::ConnectionComplete:
// Lyco HQ_5G
// reason: 0

