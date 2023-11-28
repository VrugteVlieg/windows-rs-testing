pub mod bss_entry_list;
pub mod available_network_list;
pub mod utils;
pub mod windows_api_client;
pub mod windows_type_wrappers;

use windows_api_client::WindowsApiClient;

fn main() {

        let windows_client = WindowsApiClient::init();

        let target_ssid = utils::create_dot_11_ssid("Hello World Too");
        let mut counter = 0;

        while counter < 5 {
            counter+=1;

            let full_bss_list = windows_client.retrieve_bss_list(None);
            println!("Cycle {counter}");
            let directed_bss_list = windows_client.retrieve_bss_list(Some(target_ssid));
            println!("Targeted bss list ({})\n{}\n", directed_bss_list.parsed_networks.len(), directed_bss_list);
            println!("Full bss list ({})\n{}\n", full_bss_list.parsed_networks.len(), full_bss_list);
            let full_network_list = windows_client.retrieve_network_list();
            println!("Full network list ({})\n{}\n", full_network_list.parsed_networks.len(), full_network_list);

            if counter % 2 == 0 {
                windows_client.trigger_ap_scan(Some(target_ssid));
            } else {
                windows_client.trigger_ap_scan(None);
            }

            //Scans are guaranteed to be done after 4 seconds according to the MS spec 
            //https://learn.microsoft.com/en-us/windows/win32/api/wlanapi/nf-wlanapi-wlanscan
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
}