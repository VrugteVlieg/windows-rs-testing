use windows::{Win32::{NetworkManagement::WiFi::{WLAN_AVAILABLE_NETWORK, WLAN_AVAILABLE_NETWORK_LIST, WlanGetAvailableNetworkList}, Foundation::HANDLE}, core::GUID};

use crate::utils;



pub fn get_available_network_list(handle: HANDLE, interface_guid: GUID) -> *mut WLAN_AVAILABLE_NETWORK_LIST {

    unsafe {

        let mut network_list_ptr: *mut WLAN_AVAILABLE_NETWORK_LIST = std::ptr::null_mut();
    
        WlanGetAvailableNetworkList(
            handle,
            &interface_guid,
            3,
            None,
            &mut network_list_ptr,
        );
    
        network_list_ptr
    }
    

}

#[derive(Debug)]
pub struct ParsedAvailableNetwork {
    pub ssid: String,
    pub num_bssids: u32,
    pub rssi: i32,

}

impl From<&WLAN_AVAILABLE_NETWORK> for ParsedAvailableNetwork {
    fn from(value: &WLAN_AVAILABLE_NETWORK) -> Self {
        let ssid = utils::parse_ssid(value.dot11Ssid);

        let num_bssids = value.uNumberOfBssids;
        let rssi = utils::interpolate_rssi(value.wlanSignalQuality as i32);

        ParsedAvailableNetwork { ssid, num_bssids, rssi }
    }
}

impl std::fmt::Display for ParsedAvailableNetwork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.ssid, self.num_bssids)
    }
}

#[derive(Debug)]
pub struct NetworkList {
    pub networks: Vec<WLAN_AVAILABLE_NETWORK>,
    pub parsed_networks: Vec<ParsedAvailableNetwork>
}


impl std::fmt::Display for NetworkList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "n[\n{}\n]", self.parsed_networks.iter().map(ToString::to_string).collect::<Vec<String>>().join("\n"))
    }
}



impl From<*mut WLAN_AVAILABLE_NETWORK_LIST> for NetworkList {
    fn from(value: *mut WLAN_AVAILABLE_NETWORK_LIST) -> Self {
        unsafe {

            let num_elements = (*value).dwNumberOfItems;
            let networks = utils::get_x_list_from_windows_x_list_struct::<WLAN_AVAILABLE_NETWORK_LIST, WLAN_AVAILABLE_NETWORK>(value, num_elements);
            // let first_network_ptr = (value.add(1) as *mut WLAN_AVAILABLE_NETWORK).sub(1);
            
            // let networks: Vec<WLAN_AVAILABLE_NETWORK> = Range {start: 0_usize, end: num_elements}.map(|i| {
            //     let target_network_bss = *first_network_ptr.add(i);

            //     target_network_bss
            // }).collect();

            let parsed_networks: Vec<ParsedAvailableNetwork> = networks.iter().map(ParsedAvailableNetwork::from).collect();

            NetworkList { networks, parsed_networks }
        }
    }
}