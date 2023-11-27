use std::ops::Range;

use windows::Win32::{NetworkManagement::WiFi::{WLAN_BSS_ENTRY, WLAN_BSS_LIST, WLAN_INTERFACE_INFO, DOT11_SSID, WlanGetNetworkBssList, DOT11_BSS_TYPE}, Foundation::HANDLE};

use crate::utils;


pub fn get_network_bss_list(handle: HANDLE, interface: &WLAN_INTERFACE_INFO, target_ssid: Option<*const DOT11_SSID>) -> *mut WLAN_BSS_LIST {


    let (bss_type, security_enable) = match target_ssid {
        Some(_) => {
            (1, true)   
        },
        None => (3, false)
    };

    unsafe {
        let mut network_bss_list_ptr: *mut WLAN_BSS_LIST = std::ptr::null_mut();
        WlanGetNetworkBssList(
            handle,
            &interface.InterfaceGuid,
            target_ssid,
            DOT11_BSS_TYPE(bss_type),
            security_enable,
            None,
            &mut network_bss_list_ptr,
        );

        network_bss_list_ptr
    }
}


#[derive(Debug)]
pub struct BssList {
    pub networks: Vec<WLAN_BSS_ENTRY>,
    pub parsed_networks: Vec<ParsedBssEntry>
}


impl std::fmt::Display for BssList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "[\n{}\n]", self.parsed_networks.iter().map(ToString::to_string).collect::<Vec<String>>().join("\n"))
    }
}


impl From<*mut WLAN_BSS_LIST> for BssList {
    fn from(value: *mut WLAN_BSS_LIST) -> Self {
        unsafe {
            let num_elements = (*value).dwNumberOfItems;

            let networks = utils::get_x_list_from_windows_x_list_struct::<WLAN_BSS_LIST, WLAN_BSS_ENTRY>(value, num_elements);
   
            let parsed_networks: Vec<ParsedBssEntry> = networks.iter().map(ParsedBssEntry::from).collect();

            BssList { networks, parsed_networks }
        }
    }
}
#[derive(Debug)]
pub struct ParsedBssEntry {
    pub ssid: String,
    pub bssid: String,
    pub rssi: i32,
    pub link_quality: u8,
    pub channel: u32
}

impl std::fmt::Display for ParsedBssEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} @ {}  ({})", self.ssid, self.bssid, self.rssi)
    }
}

impl From<&WLAN_BSS_ENTRY> for ParsedBssEntry {
    fn from(value: &WLAN_BSS_ENTRY) -> Self {
        let ssid = utils::parse_ssid(value.dot11Ssid);

        let bssid = utils::parse_bssid(value.dot11Bssid);

        let rssi = value.lRssi;
        let link_quality = value.uLinkQuality as u8;
        let channel = utils::map_freq_to_channel(value.ulChCenterFrequency);
        ParsedBssEntry { ssid, bssid, rssi, link_quality, channel }
    }
}
