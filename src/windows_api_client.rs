use windows::Win32::{NetworkManagement::WiFi::{WlanOpenHandle, WLAN_INTERFACE_INFO_LIST, L2_NOTIFICATION_DATA, WLAN_INTERFACE_INFO, WlanEnumInterfaces, WlanRegisterNotification, WlanGetAvailableNetworkList, WLAN_AVAILABLE_NETWORK_LIST, WlanGetNetworkBssList, DOT11_BSS_TYPE, WLAN_BSS_LIST, WlanCloseHandle, WlanScan, DOT11_SSID}, Foundation::HANDLE};

use crate::{utils::{self}, available_network_list::NetworkList, bss_entry_list::BssList};

use crate::windows_type_wrappers::{WlanNotifcationSource, AcmNotifcationType, OnexNotifcationType, HostedNetworkNoticationType, MsmNotifcationType};



struct WindowsNotifcation {
    notifcation_source: WlanNotifcationSource,
    notifcation_reason: i32,
    notifcation_string: String
}

pub struct WindowsApiClient {
    handle: HANDLE,
    network_interface: WLAN_INTERFACE_INFO,
}

unsafe extern "system" fn notif_callback(param0: *mut L2_NOTIFICATION_DATA, _param1: *mut ::core::ffi::c_void) {
    let notifcation_data = *param0;
    println!("Callback triggerd: {:?}", notifcation_data);
    let notif_reason = WlanNotifcationSource::try_from(notifcation_data.NotificationSource).unwrap();
    println!("Reason: {:?}", notif_reason);
    let notif_code = notifcation_data.NotificationCode as i32;
    let notifcation_type = match notif_reason {
        WlanNotifcationSource::ACM => Some(format!("{:?}", AcmNotifcationType::try_from(notif_code).unwrap())),
        WlanNotifcationSource::ONEX => Some(format!("{:?}", OnexNotifcationType::try_from(notif_code).unwrap())),
        WlanNotifcationSource::HNWK => Some(format!("{:?}", HostedNetworkNoticationType::try_from(notif_code).unwrap())),
        // https://stackoverflow.com/questions/63916457/wlan-notification-msm-notificationcode-59
        WlanNotifcationSource::MSM if notif_code != 59 => Some(format!("{:?}", MsmNotifcationType::try_from(notif_code).unwrap())),
        _ => None
    };
    if let Some(notification_string) = notifcation_type {
        println!("Notifcation String: {}", notification_string);
    }
}


impl WindowsApiClient {
    pub fn init() -> Self {
        let mut handle: HANDLE = HANDLE::default();
        let mut client_version: u32 = 0;
        let mut interface_list_ptr: *mut WLAN_INTERFACE_INFO_LIST = std::ptr::null_mut();
        unsafe {

            WlanOpenHandle(
                2,
                None,
                &mut client_version as *mut u32,
                &mut handle as *mut HANDLE,
            );

            WlanEnumInterfaces(handle, None, &mut interface_list_ptr);
            let network_interfaces: Vec<WLAN_INTERFACE_INFO> = utils::get_x_list_from_windows_x_list_struct::<WLAN_INTERFACE_INFO_LIST, WLAN_INTERFACE_INFO>(interface_list_ptr, (*interface_list_ptr).dwNumberOfItems);

            //https://learn.microsoft.com/en-us/windows/win32/api/wlanapi/nf-wlanapi-wlanregisternotification
            //Related to WlanNotifcationSource in windows_type_wrappers but the documentation treats them as separate types so I keep them separate
            let notification_source_all = 65535u32;
            
            // https://learn.microsoft.com/en-us/windows/win32/api/wlanapi/nc-wlanapi-wlan_notification_callback
            // We could potentially pass a pointer to some a Sender to publish notifcations when we receive them, haven't looked into this properly
            let callback_context = None;

            WlanRegisterNotification(handle, notification_source_all, false, Some(notif_callback), callback_context, None, None);

            WindowsApiClient { handle, network_interface: *network_interfaces.first().unwrap()}
        }

    }

    pub fn retrieve_network_list(&self) -> NetworkList {
        unsafe {

            let mut network_list_ptr: *mut WLAN_AVAILABLE_NETWORK_LIST = std::ptr::null_mut();
        
            println!("Fetching networks list");
            //This returns duplicates for networks that you have already connected to before, the networks that have a profile
            //https://github.com/jorgebv/windows-wifi-api/issues/7
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

    pub fn retrieve_bss_list(&self, target_ssid: Option<DOT11_SSID>) -> BssList {
        unsafe {
            let (bss_type, security_enable) = match target_ssid {
                Some(_) => {
                    (1, true)   
                },
                None => (3, false)
            };

            let mut network_bss_list_ptr: *mut WLAN_BSS_LIST = std::ptr::null_mut();

            if let Some(target_ssid) = target_ssid {
                let struct_ptr: *const DOT11_SSID = &target_ssid;
                WlanGetNetworkBssList(
                    self.handle,
                    &self.network_interface.InterfaceGuid,
                    Some(struct_ptr),
                    DOT11_BSS_TYPE(bss_type),
                    security_enable,
                    None,
                    &mut network_bss_list_ptr,
                );
            } else {
                WlanGetNetworkBssList(
                    self.handle,
                    &self.network_interface.InterfaceGuid,
                    None,
                    DOT11_BSS_TYPE(bss_type),
                    security_enable,
                    None,
                    &mut network_bss_list_ptr,
                );
            }
  
            let results = BssList::from(network_bss_list_ptr);

            if let Some(target_ssid) = target_ssid {
                let ssid_str = utils::parse_ssid(target_ssid);
                println!("Results from targeted scan for {}:\n{:?}", ssid_str, results);
            }
            
         
            results
        }
    }

    pub fn trigger_ap_scan(&self, target_ssid: Option<DOT11_SSID>) {
        unsafe {
            if let Some(target_ssid) = target_ssid {
                println!("Triggering targeted ap scan for {}", utils::parse_ssid(target_ssid));
                let struct_ptr: *const DOT11_SSID = &target_ssid;
                WlanScan(self.handle, &self.network_interface.InterfaceGuid, Some(struct_ptr), None, None);
            } else {
                WlanScan(self.handle, &self.network_interface.InterfaceGuid, None, None, None);
            }
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