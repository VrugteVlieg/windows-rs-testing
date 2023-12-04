pub mod utils;
pub mod windows_api_client;
pub mod windows_type_wrappers;
pub mod windows_data_collector;

use utils::NetworkBand;
use windows::Win32::NetworkManagement::WiFi::{WLAN_AVAILABLE_NETWORK, WLAN_BSS_ENTRY};
use windows_api_client::WindowsApiClient;


#[tokio::main]
async fn main() {

        WindowsApiClient::init();

        let target_ssid = utils::create_dot_11_ssid("Hello World Too");
        let mut counter = 0;

        loop {
            println!("Starting cycle {counter}");
            let directed_scan = counter % 2 == 0;
            counter+=1;

            
            let result = WindowsApiClient::ap_scan(if directed_scan {Some(target_ssid)} else {None}).await;
            
            println!("All networks:\n{}", result.iter().map(ToString::to_string).collect::<Vec<String>>().join("\n"));
            

            //Scans are guaranteed to be done after 4 seconds according to the MS spec 
            //https://learn.microsoft.com/en-us/windows/win32/api/wlanapi/nf-wlanapi-wlanscan
            std::thread::sleep(std::time::Duration::from_secs(3));
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