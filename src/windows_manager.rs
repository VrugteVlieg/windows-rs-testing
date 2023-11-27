use std::sync::mpsc::Receiver;

use windows::Win32::{NetworkManagement::WiFi::{WlanOpenHandle, WLAN_INTERFACE_INFO_LIST, L2_NOTIFICATION_DATA, WLAN_INTERFACE_INFO, WlanEnumInterfaces, WlanRegisterNotification, WlanGetAvailableNetworkList, WLAN_AVAILABLE_NETWORK_LIST, WlanGetNetworkBssList, DOT11_BSS_TYPE, WLAN_BSS_LIST, WlanCloseHandle, WlanScan, DOT11_SSID}, Foundation::HANDLE};

use crate::{utils::{self, WlanNotifcationSource}, available_network_list::NetworkList, bss_entry_list::{BssList, get_network_bss_list}};



struct WindowsNotifcation {
    notifcation_source: WlanNotifcationSource,
    notifcation_reason: i32
}

pub struct WindowsApiClient {
    handle: HANDLE,
    network_interface: WLAN_INTERFACE_INFO,
    // events_receiver: Option<Receiver<WindowsNotifcation>>,
    stale_network_list: bool
}

// pub fn create_windows_api_client() {
//     GLOBAL_WINDOWS_API_CLIENT::
// }



unsafe extern "system" fn notif_callback(param0: *mut L2_NOTIFICATION_DATA, _param1: *mut ::core::ffi::c_void) {
    let notifcation_data = *param0;
    println!("Callback triggerd: {:?}", notifcation_data);
    let notif_reason = utils::WlanNotifcationSource::try_from(notifcation_data.NotificationSource).unwrap();
    println!("Reason: {:?}", notif_reason);
    let notif_code = notifcation_data.NotificationCode as i32;
    let notifcation_type = match notif_reason {
        utils::WlanNotifcationSource::ACM => Some(format!("{:?}", utils::AcmNotifcationType::try_from(notif_code).unwrap())),
        utils::WlanNotifcationSource::ONEX => Some(format!("{:?}", utils::OnexNotifcationType::try_from(notif_code).unwrap())),
        utils::WlanNotifcationSource::HNWK => Some(format!("{:?}", utils::HostedNetworkNoticationType::try_from(notif_code).unwrap())),
        // https://stackoverflow.com/questions/63916457/wlan-notification-msm-notificationcode-59
        utils::WlanNotifcationSource::MSM if notif_code != 59 => Some(format!("{:?}", utils::MsmNotifcationType::try_from(notif_code).unwrap())),
        _ => None
    };
    if let Some(notification_string) = notifcation_type {
        println!("Notifcation code: {}", notification_string);
    }
}


impl WindowsApiClient {
    pub fn init() -> Self {
        let mut handle: HANDLE = HANDLE::default();
        let mut client_version: u32 = 0;
        let mut interface_list_ptr: *mut WLAN_INTERFACE_INFO_LIST = std::ptr::null_mut();
        unsafe {

            println!("Create wlan handle");
            WlanOpenHandle(
                2,
                None,
                &mut client_version as *mut u32,
                &mut handle as *mut HANDLE,
            );

            println!("Enumerating interfaces");
            WlanEnumInterfaces(handle, None, &mut interface_list_ptr);
            let network_interfaces: Vec<WLAN_INTERFACE_INFO> = utils::get_x_list_from_windows_x_list_struct(interface_list_ptr, (*interface_list_ptr).dwNumberOfItems);
    
            println!("Registering callback");
            WlanRegisterNotification(handle, 65535u32, false, Some(notif_callback), None, None, None);

            println!("Api client creation success");
            WindowsApiClient { handle, network_interface: *network_interfaces.first().unwrap(), stale_network_list: true}
        }

    }

    pub fn retrieve_network_list(self) -> NetworkList {
        unsafe {

            let mut network_list_ptr: *mut WLAN_AVAILABLE_NETWORK_LIST = std::ptr::null_mut();
        
            println!("Fetching networks list");
            WlanGetAvailableNetworkList(
                self.handle,
                &self.network_interface.InterfaceGuid,
                3,
                None,
                &mut network_list_ptr,
            );
        
            NetworkList::from(network_list_ptr)        
        }
    }

    pub fn retrieve_bss_list(&self, target_ssid: Option<*const DOT11_SSID>) -> BssList {
        unsafe {
            let (bss_type, security_enable) = match target_ssid {
                Some(_) => {
                    (1, true)   
                },
                None => (3, false)
            };

            let target_ssid_str = if let Some(target_ssid) = target_ssid.clone() {
                utils::parse_ssid((*target_ssid).clone())      
            } else {
                "None".to_string()
            };
            println!("Fetching networks bss list with direction {target_ssid_str}", );
            // let p_dot_11_ssid = target_ssid.map(|e| utils::create_dot_11_ssid_ptr(&e));
            let mut network_bss_list_ptr: *mut WLAN_BSS_LIST = std::ptr::null_mut();


            println!("Pdot11SSID: {:?}", target_ssid);

            // BssList::from(get_network_bss_list(self.handle, &self.network_interface, p_dot_11_ssid))
            
            WlanGetNetworkBssList(
                self.handle,
                &self.network_interface.InterfaceGuid,
                target_ssid,
                DOT11_BSS_TYPE(bss_type),
                security_enable,
                None,
                &mut network_bss_list_ptr,
            );
        
            let results = BssList::from(network_bss_list_ptr);
            if target_ssid.is_some() && !results.parsed_networks.is_empty() {
                let output = BssList {
                    networks: results.networks,
                    parsed_networks: 
                }
                
            }
            out
        }
    }

    pub fn trigger_ap_scan(&self, target_ssid: Option<*const DOT11_SSID>) {
        println!("Triggering {} ap scan", if target_ssid.is_some() {"directed"} else {"undirected"});
        unsafe {
            if let Some(ref target) = target_ssid {
                println!("SSID has len {}\nbytes: {:?}", (**target).uSSIDLength, (**target).ucSSID);
            }
            WlanScan(self.handle, &self.network_interface.InterfaceGuid, target_ssid, None, None);
        }

    }
}


impl Drop for WindowsApiClient {
    fn drop(&mut self) {
        unsafe {
            WlanCloseHandle(self.handle, None);
        }
    }
}