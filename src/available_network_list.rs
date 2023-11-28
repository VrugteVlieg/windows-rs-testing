use windows::Win32::NetworkManagement::WiFi::{WLAN_AVAILABLE_NETWORK, WLAN_AVAILABLE_NETWORK_LIST};

use crate::utils;


#[derive(Debug)]
pub struct ParsedAvailableNetwork {
    pub ssid: String,
    pub num_bssids: u32,
    pub rssi: i32,
    pub network_security: Option<String>,

}

impl From<&WLAN_AVAILABLE_NETWORK> for ParsedAvailableNetwork {
    fn from(value: &WLAN_AVAILABLE_NETWORK) -> Self {
        let ssid = utils::parse_ssid(value.dot11Ssid);

        let num_bssids = value.uNumberOfBssids;
        let rssi = utils::interpolate_rssi(value.wlanSignalQuality as i32);

        let network_security = if value.bSecurityEnabled.as_bool() {
            Some(format!("{:?} - {:?}", value.dot11DefaultAuthAlgorithm, value.dot11DefaultCipherAlgorithm))
        } else {
            None
        };


        ParsedAvailableNetwork { ssid, num_bssids, rssi, network_security }
    }
}

impl std::fmt::Display for ParsedAvailableNetwork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({}) auth: {:?}", self.ssid, self.num_bssids, self.network_security)
    }
}

#[derive(Debug)]
pub struct NetworkList {
    pub networks: Vec<WLAN_AVAILABLE_NETWORK>,
    pub parsed_networks: Vec<ParsedAvailableNetwork>
}


impl std::fmt::Display for NetworkList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "[\n{}\n]", self.parsed_networks.iter().map(ToString::to_string).collect::<Vec<String>>().join("\n"))
    }
}



impl From<*mut WLAN_AVAILABLE_NETWORK_LIST> for NetworkList {
    fn from(value: *mut WLAN_AVAILABLE_NETWORK_LIST) -> Self {
        unsafe {

            let num_elements = (*value).dwNumberOfItems;
            let networks = utils::get_x_list_from_windows_x_list_struct::<WLAN_AVAILABLE_NETWORK_LIST, WLAN_AVAILABLE_NETWORK>(value, num_elements);
     
            let parsed_networks: Vec<ParsedAvailableNetwork> = networks.iter().map(ParsedAvailableNetwork::from).collect();

            NetworkList { networks, parsed_networks }
        }
    }
}