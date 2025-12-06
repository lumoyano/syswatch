use std::fmt::{write, Formatter};
use std::time::Duration;
use anyhow::Result;
use futures::future::join_all;
use get_if_addrs::{get_if_addrs, IfAddr};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::process::Stdio;
use tokio::process::Command;
use tokio::net::TcpStream;
use tokio::time::timeout;

//Enum states, use derive so it can be copied and whatnot
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentState {
    Up,
    Degraded,
    Down,
    Unknown,
}
//Standard way of formatting enums from the standard format display library
//only the inside matters/changes - basically a tostring
impl std::fmt::Display for ComponentState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentState::Up => {write!(f, "Up")}
            ComponentState::Degraded => {write!(f, "Degraded")}
            ComponentState::Down => {write!(f, "Down")}
            ComponentState::Unknown => {write!(f, "Unknown")}
        }
    }
}

//summary ish of the diagnosis struct
#[derive(Debug)]
pub struct NetDiagnosis {
    pub interface_up: ComponentState,
    pub wifi_on: ComponentState,
    pub ssid: Option<String>,
    pub signal_dbm: Option<i16>,
    pub ap_reachable: ComponentState,
    pub gateway_reachable: ComponentState,
    pub lan_hosts_reachable: usize,
    pub dns_ok: ComponentState,
    pub external_connect_ms: Option<u128>,
    pub http_ok: ComponentState,
    pub passive_bytes_seen: Option<u64>,
    pub confidence: f32, // 0.0..1.0
    pub notes: Vec<String>,
}

//main and public entry function
pub async fn scan_network(gateway: Option<Ipv4Addr>, subnet: Option<String>) -> Result<()> {
    let diag = diagnose(gateway, subnet).await?;
    print_report(&diag);
    Ok(())
}

pub async fn diagnose(gateway: Option<Ipv4Addr>, subnet: Option<String>) -> anyhow::Result<NetDiagnosis> {
    //timeout parameters
    let local_timeout = Duration::from_millis(400);
    let external_timeout = Duration::from_secs(2);
    let http_timeout = Duration::from_secs(4);

    //====================check interface=======================
    let if_info = collect_interfaces()?;
    let interface_up = if !if_info.is_empty() {
        ComponentState::Up
    } else {
        ComponentState::Down
    };

    //====================check Wifi info=======================
    let (wifi_on, ssid, signal_dbm) = detect_wifi().await;

    // If no interface, skip the rest and print a dummy result
    if interface_up == ComponentState::Down {
        let mut d = print_interface_down(interface_up, wifi_on, ssid, signal_dbm);
        if wifi_on == ComponentState::Down {
            d.notes.push("No Wi‑Fi interface or radio appears off".into());
        }
        return Ok(d);
    }

    //====================Check Gateway=======================
    let mut notes = Vec::new();
    let gateway_reachable = if let Some(gw) = gateway {
        let addr = SocketAddr::new(IpAddr::V4(gw), 80);
        match probe_tcp(addr, local_timeout).await? {
            true => ComponentState::Up,
            false => {
                notes.push(format!("Gateway {} not reachable on port 80", gw));
                ComponentState::Down
            }
        }
    } else {
        notes.push("No gateway provided".into());
        ComponentState::Unknown
    };

    // optional: scan subnet if provided
    let mut lan_hosts_reachable = 0;
    if let Some(subnet_str) = subnet {
        if let Ok(net) = subnet_str.parse::<ipnetwork::Ipv4Network>() {
            for ip in net.iter() {
                if Some(ip) == gateway {
                    continue;
                }
                let addr = SocketAddr::new(IpAddr::V4(ip), 80);
                if probe_tcp(addr, local_timeout).await? {
                    lan_hosts_reachable += 1;
                }
            }
        } else {
            notes.push(format!("Invalid subnet string: {}", subnet_str));
        }
    }

    // build diagnosis result
    let d = NetDiagnosis {
        interface_up,
        wifi_on,
        ssid,
        signal_dbm,
        ap_reachable: gateway_reachable, // treat gateway as AP proxy
        gateway_reachable,
        lan_hosts_reachable,
        dns_ok: ComponentState::Unknown, // not yet implemented
        external_connect_ms: None,
        http_ok: ComponentState::Unknown,
        passive_bytes_seen: None,
        confidence: 0.8,
        notes,
    };

    Ok(d)
}

fn print_interface_down(interface_up: ComponentState, wifi_on: ComponentState, ssid: Option<String>, signal_dbm: Option<i16>) -> NetDiagnosis {
    let mut d = NetDiagnosis {
        interface_up,
        wifi_on,
        ssid,
        signal_dbm,
        ap_reachable: ComponentState::Unknown,
        gateway_reachable: ComponentState::Down,
        lan_hosts_reachable: 0,
        dns_ok: ComponentState::Unknown,
        external_connect_ms: None,
        http_ok: ComponentState::Unknown,
        passive_bytes_seen: None,
        confidence: 0.9,
        notes: vec!["No non-loopback interfaces with addresses detected".into()],
    };
    d
}

/// Collect non-loopback interface addresses (best-effort)
fn collect_interfaces() -> Result<Vec<IpAddr>> {
    let mut addrs = Vec::new();
    for iface in get_if_addrs()? {
        match iface.addr {
            IfAddr::V4(v4) => {
                if !v4.ip.is_loopback() {
                    addrs.push(IpAddr::V4(v4.ip));
                }
            }
            IfAddr::V6(v6) => {
                if !v6.ip.is_loopback() {
                    addrs.push(IpAddr::V6(v6.ip));
                }
            }
        }
    }
    Ok(addrs)
}

/// Platform-aware Wi‑Fi detection
/// - Linux: uses iwgetid/iwconfig or /proc/net/wireless
/// - Windows: uses `netsh wlan show interfaces` and parses SSID + Signal
/// - Other platforms: Unknown
async fn detect_wifi() -> (ComponentState, Option<String>, Option<i16>) {
    #[cfg(target_os = "windows")]
    {
        // Use netsh to get SSID and signal percent
        let mut ssid: Option<String> = None;
        let mut signal_pct: Option<i16> = None;

        let out = Command::new("netsh")
            .args(&["wlan", "show", "interfaces"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .await;

        if let Ok(o) = out {
            let txt = String::from_utf8_lossy(&o.stdout);
            for line in txt.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("SSID") && trimmed.contains(":") {
                    // avoid "BSSID"
                    if trimmed.starts_with("SSID") && !trimmed.starts_with("BSSID") {
                        if let Some(idx) = trimmed.find(':') {
                            let val = trimmed[idx + 1..].trim();
                            if !val.is_empty() && val != "not available" {
                                ssid = Some(val.to_string());
                            }
                        }
                    }
                } else if trimmed.starts_with("Signal") && trimmed.contains(':') {
                    if let Some(idx) = trimmed.find(':') {
                        let val = trimmed[idx + 1..].trim().trim_end_matches('%').trim();
                        if let Ok(p) = val.parse::<i16>() {
                            signal_pct = Some(p);
                        }
                    }
                }
            }
        }

        // If we found either, mark Wi‑Fi Up
        let wifi_on = if ssid.is_some() || signal_pct.is_some() {
            ComponentState::Up
        } else {
            ComponentState::Unknown
        };

        // Convert percent to rough dBm estimate
        // 100% -> -50 dBm, 50% -> -75 dBm, 0% -> -100 dBm
        let signal_dbm = signal_pct.map(|pct| {
            let pct = pct.clamp(0, 100) as i32;
            // linear interpolation between -100 and -50
            let dbm = -100 + ((pct as i32 * 50) / 100);
            dbm as i16
        });

        (wifi_on, ssid, signal_dbm)
    }
}

// Probe a TCP socket with a timeout; returns true if connect succeeded
async fn probe_tcp(addr: SocketAddr, to: Duration) -> Result<bool> {
    match timeout(to, TcpStream::connect(addr)).await {
        Ok(Ok(_)) => Ok(true),
        _ => Ok(false),
    }
}

fn print_report(diag: &NetDiagnosis) {
    println!("=== Network Diagnosis Report ===");
    println!("Interface: {}", diag.interface_up);
    println!("Wi‑Fi: {}", diag.wifi_on);

    if let Some(ssid) = &diag.ssid {
        println!("SSID: {}", ssid);
    }
    if let Some(signal) = diag.signal_dbm {
        println!("Signal Strength: {} dBm", signal);
    }

    println!("Access Point Reachable: {}", diag.ap_reachable);
    println!("Gateway Reachable: {}", diag.gateway_reachable);
    println!("LAN Hosts Reachable: {}", diag.lan_hosts_reachable);
    println!("DNS OK: {}", diag.dns_ok);

    if let Some(ms) = diag.external_connect_ms {
        println!("External Connect Time: {} ms", ms);
    }
    println!("HTTP OK: {}", diag.http_ok);

    if let Some(bytes) = diag.passive_bytes_seen {
        println!("Passive Bytes Seen: {}", bytes);
    }

    println!("Confidence: {:.1}", diag.confidence);

    if !diag.notes.is_empty() {
        println!("Notes:");
        for note in &diag.notes {
            println!("  - {}", note);
        }
    }

    println!("================================");
}
