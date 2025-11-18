# Overview

Working with a subpar computer and less than appropriate internet have been my personal demons.
This tool aims to aid me in my struggles and ideally gives some insight into what is actually lacking.

I's calling this Version 1.2 of the syswatch program, specifically because I feel as if I only got less than half 
of the expected functionality done over this period of time.

[Software Demo Video](https://youtu.be/OSjrKQI2y9w)

# Changelog
** November 18th, 2025 - v1.2**
- Multiple flags supported
- Net flag functionality partially implemented
  - Check if interface is up
  - Find connection and SSID
  - Approximate signal strength
  - Test AP and Router
  - Gateway and Subnet Options (Required with net function for now)
- All of Main is now async 

# Development Environment

This was all written in Rustover by JetBrains as they're my preferred IDE in literally anything ever.
I used dependencies (or Crates for those who get it) like Clap for quick building of CLI tools, WinReg to fetch Windows 
Registry Words, and SysInfo to fetch live info from the machine.

The Project has grown to use Tokio for asynchronous functionality, Futures, AnyHow, Ipnetwork, to handle networking 
elements from this update.

Special note to the use of LLMs. I've use Copilot rather than chatGPT because it's a lot more "free" in the sense that 
it's less limiting, and can handle pictures without limit

# Useful Websites

Some of the websites I've relied on thus far: 

- [Crates.io](https://crates.io/crates/sysinfo/0.30.13)
- [W3 Schools Rust Tutorial](https://www.w3schools.com/)
- [Sysinfo Documentation](https://docs.rs/sysinfo/latest/sysinfo/struct.System.html)
- [Tokio Documentation](https://docs.rs/tokio/latest/tokio/index.html)
- [StackOverflow (Multiple threads)](https://stackoverflow.com/questions)

# Future Work

To finish the network tool there is still a lot of work. I still need to implement all the items in the Result struct,
E.g. Check visible Hosts, DNS and HTTP.

- **New TODO** Finish Result Struct item discovery
- **New TODO** Find device specific info for at least AP and Router when present
- **New TODO** Redesign Confidence system (or get rid of it)
- **New TODO** Store results in JSON format
- **New TODO** Run as background service (it would be a different scope of work)
- **New TODO** Fix units in top processes
- Live data readings (similar to linux "top")
- Long form information about hardware
- Full length guides on Windows performance improvements
- Ability to implement suggested registry words directly from the CLI