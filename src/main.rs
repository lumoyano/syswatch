use clap::Parser;
use sysinfo::Disks;

#[derive(Parser, Debug)]
#[command(name="syswatch")]
#[command(version="1")]
#[command(about="Basic tool to offer system insights on usage and suggestions to improve performance")]
struct Flags {
    #[arg(short, long)]
    net: bool, //show whether we are connected, the network info, and status
    #[arg(short, long)]
    long: bool, //short-form summary of everything
    #[arg(short, long)]
    win: bool //check for possible enhancements to windows (like disabling bing on windows searchbar
}
fn main() {
    let flags = Flags::parse();

    // Count how many flags were selected
    let selected_count = flags.net as u8 + flags.long as u8 + flags.win as u8;

    if selected_count == 0 {
        run_default();
        return;
    }

    if selected_count > 1 {
        println!("Only one option can be used at a time.\n");
        show_usage();
        return;
    }

    match () {
        _ if flags.long => run_long(),
        _ if flags.net => run_network_info(),
        _ if flags.win => run_windows_check(),
        _ => unreachable!()
    }
}

fn show_usage() {
    println!("Usage:");
    println!("Calling syswatch with no flags will default to a short version of everything");
    println!("  syswatch --long   Show longform system information");
    println!("  syswatch --net     Show network connectivity + info");
    println!("  syswatch --win     Check Windows UI/privacy improvements");
}

fn run_long() {
    println!("(placeholder) System summary coming soon...");
}

fn run_network_info() {
    println!("(placeholder) Network details coming soon...");
}

fn run_windows_check() {
    use winreg::enums::*;
    use winreg::RegKey;

    println!("Windows recommended settings:");

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    //Bing in windows search, cortana search results
    let search_key = hkcu.open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Search");
    match search_key {
        Ok(key) => {
            let bing: Result<u32, _> = key.get_value("BingSearchEnabled");
            let cortana: Result<u32, _> = key.get_value("CortanaConsent");

            println!("RECOMMENDATION: Disable Bing web search and Cortana from Windows Search:");
            match (bing, cortana) {
                (Ok(0), Ok(0)) => println!("     ✅ Done"),
                _ => println!("     ❌ Bing web search or Cortana is still enabled."),
            }
        }
        Err(_) => println!("⚠ Could not read Search settings."),
    }

    // ----- Check News & Interests / Widgets -----
    let adv_key = hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced");
    match adv_key {
        Ok(key) => {
            let taskbar_da: Result<u32, _> = key.get_value("TaskbarDa");
            println!("RECOMMENDATION: Disable News and Interests menu:");
            match taskbar_da {
                Ok(0) => println!("     ✅ News & Interests (weather/news hover menu) is disabled."),
                _ => println!("     ❌ News & Interests is still enabled."),
            }
        }
        Err(_) => println!("⚠ Could not read taskbar settings."),
    }
}

fn run_default() {
    use sysinfo::{System};
    use std::net::TcpStream;

    let mut sys = System::new_all();
    let dsks = Disks::new_with_refreshed_list();
    sys.refresh_all();

    // CPU Usage
    let avg_cpu = sys.cpus()
        .iter()
        .map(|cpu| cpu.cpu_usage())
        .sum::<f32>() / sys.cpus().len() as f32;
    println!("CPU Usage: {:.1}%", avg_cpu);

    // Memory Usage
    let total_mem = sys.total_memory() / 1024;
    let used_mem = sys.used_memory() / 1024;
    println!("Memory Usage: {}/{} MB ({:.1}%)",
             used_mem,
             total_mem,
             used_mem as f32 / total_mem as f32 * 100.0
    );

    // Disk Usage (only first partition)
    if let Some(disk) = dsks.get(0) {
        let total = disk.total_space() / 1024 / 1024 / 1024;
        let avail = disk.available_space() / 1024 / 1024 / 1024;
        println!("Disk: {} GB free / {} GB total", avail, total);
    }

    // Network Connectivity Test
    let connected = TcpStream::connect("8.8.8.8:53").is_ok();
    println!("Network: {}", if connected { "✅ Online" } else { "❌ Offline" });

    // Top 5 CPU-heavy processes
    println!("\nTop Processes (by CPU):");
    let mut procs: Vec<_> = sys.processes().values().collect();
    procs.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap());

    for process in procs.into_iter().take(5) {
        println!(
            "{:<30} CPU: {:>5.1}%   Memory: {} MB",
            process.name(),
            process.cpu_usage(),
            process.memory() / 1024
        );
    }
}