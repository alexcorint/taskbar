// Red: estado de conectividad y tipo de conexión.

use crate::types::NetworkStatus;
use windows::Networking::Connectivity::NetworkInformation;

pub fn get_network_status() -> NetworkStatus {
    let mut status = NetworkStatus {
        is_online: false,
        connection_type: "none".to_string(),
        signal_strength: 0,
    };

    if let Ok(profile) = NetworkInformation::GetInternetConnectionProfile() {
        if let Ok(level) = profile.GetNetworkConnectivityLevel() {
            status.is_online = level.0 >= 3; // 3 = InternetAccess
        }

        if let Ok(adapter) = profile.NetworkAdapter() {
            if let Ok(iana_type) = adapter.IanaInterfaceType() {
                status.connection_type = match iana_type {
                    71 => "wifi".to_string(),
                    6 => "ethernet".to_string(),
                    _ => "unknown".to_string(),
                };
            }
        }

        if let Ok(bars) = profile.GetSignalBars() {
            if let Ok(val) = bars.Value() {
                status.signal_strength = val;
            }
        }
    }

    status
}
