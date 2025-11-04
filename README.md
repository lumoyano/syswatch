# Overview

Working with a subpar computer and less than appropriate internet have been my personal demons.
This tool aims to aid me in my struggles and ideally gives some insight into what is actually lacking.

Version 1.0 of syswatch is a CLI tool with the ability to show information on the system performance, and will be expanded to handle mostly the network as well.

[Software Demo Video](https://youtu.be/gI9Du4kREpY)

# Development Environment

This was all written in Rustover by JetBrains as they're my preferred IDE in literally anything ever.
I used dependencies (or Crates for those who get it) like Clap for quick building of CLI tools, WinReg to fetch Windows Registry Words,
and SysInfo to fetch live info from the machine.

Special note to the use of LLMs. Mine was ChatGPT almost exclusively as far as version 1.0 goes, and you will find that any 
print formatting as well as specific types such as u32 or f64 explicitly typed was copied and pasted from it.
# Useful Websites

Some of the websites I've relied on thus far: 

- [Crates.io](https://crates.io/crates/sysinfo/0.30.13)
- [W3 Schools Rust Tutorial](https://www.w3schools.com/)
- [Sysinfo Documentation](https://docs.rs/sysinfo/latest/sysinfo/struct.System.html)

# Future Work

The next step on the Software will be to create the network tool, which is what I would like this to be most useful for.
Some other items come to mind in no order necessarily

- Live data readings (similar to linux "top")
- Long form information about hardware
- Full length guides on Windows performance improvements
- Ability to implement suggested registry words directly from the CLI