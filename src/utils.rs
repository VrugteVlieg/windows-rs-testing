use std::ops::Range;

use windows::Win32::NetworkManagement::WiFi::DOT11_SSID;
use anyhow::anyhow;

pub fn map_freq_to_channel(freq: u32) -> u32 {
    let lower_bound_5_ghz = 5160_000;
    let lower_bound_2_ghz = 2412_000;
    let channel_center_separation = 5_000;

    if freq > lower_bound_5_ghz {
        32 + (freq - lower_bound_5_ghz) / channel_center_separation
    } else {
        1 + (freq - lower_bound_2_ghz) / channel_center_separation
    }
}

//convert the signal percentage to to dbm according to https://learn.microsoft.com/en-us/windows/win32/api/wlanapi/ns-wlanapi-wlan_available_network
pub fn interpolate_rssi(x: i32) -> i32 {
    let (x0, x1) = (0, 100);
    let (y0, y1) = (-100, -50);
    y1 + ((x - x1) * (y1 - y0) / (x1 - x0))
}

pub fn parse_ssid(input: DOT11_SSID) -> String {
    let ssid_len = input.uSSIDLength as usize;
    let ssid = String::from_utf8(input.ucSSID[0..ssid_len].to_vec())
        .unwrap_or("Unable to parse ssid".to_string());

    // println!("Parsed {} from {:#?}", ssid, input);

    return ssid;
}

pub fn parse_bssid(input: [u8; 6]) -> String {
    input.map(|e| format!("{e:02X}")).join(":")
}


pub fn create_dot_11_ssid_ptr(ssid: &str) -> *const DOT11_SSID {
    println!("Creating ssid struct from {ssid}");
    let mut ssid_buffer = [0_u8; 32];
    let ssid_len = ssid.len();
    ssid_buffer[0..ssid_len].copy_from_slice(ssid.as_bytes());
    let dot11_ssid_ptr: *const DOT11_SSID = &DOT11_SSID {
        uSSIDLength: ssid_len as u32,
        ucSSID: ssid_buffer
    };
    unsafe {
        println!("Created struct ptr @ {:p} {:?} ", dot11_ssid_ptr, *dot11_ssid_ptr);
    }

    dot11_ssid_ptr
}


pub fn create_dot_11_ssid(ssid: &str) -> DOT11_SSID {
    println!("Creating ssid struct from {ssid}");
    let mut ssid_buffer = [0_u8; 32];
    let ssid_len = ssid.len();
    ssid_buffer[0..ssid_len].copy_from_slice(ssid.as_bytes());
    let dot11_ssid = DOT11_SSID {
        uSSIDLength: ssid_len as u32,
        ucSSID: ssid_buffer
    };
    
    println!("Created struct {:?} ", dot11_ssid);
    

    dot11_ssid
}




pub fn get_x_list_from_windows_x_list_struct<XListStruct, X: Copy>(list_ptr: *mut XListStruct, num_elements: u32) -> Vec<X> {
    unsafe {
        let base_pointer = (list_ptr.add(1) as *mut X).sub(1);
        let x_list: Vec<X> = Range {start: 0, end: num_elements}.map(|i| {
            let target_struct = *base_pointer.add(i as usize);

            target_struct
        }).collect();
        x_list
    }
}