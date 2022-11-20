use std::env;
use std::fs;
use users;
use users::os::unix::UserExt;

fn find_line_starting_with<'a>(text: &'a str, pfx: &str) -> Option<&'a str> {
    text.lines().find(|l| l.starts_with(pfx))
}

fn find_i64_in_line_starting_with(text: &str, pfx: &str) -> Option<i64> {
    find_line_starting_with(text, pfx).and_then(|s| {
        s.chars()
            .skip(pfx.len())
            .skip_while(|c| !('0' <= *c && *c <= '9'))
            .take_while(|c| '0' <= *c && *c <= '9')
            .collect::<String>()
            .parse::<i64>()
            .ok()
    })
}

fn argvec_asks_for_help() -> bool {
    let args = env::args().collect::<Vec<String>>();
    args.contains(&String::from("-h"))
        || args.contains(&String::from("/h"))
        || args.contains(&String::from("/help"))
        || args.contains(&String::from("--help"))
        || args.contains(&String::from("/?"))
        || args.contains(&String::from("-?"))
}

enum ComparationMode {
    RAM,
    SWAP,
    COMB,
}

impl TryFrom<&str> for ComparationMode {
    type Error = std::io::ErrorKind;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "RAM" => Ok(Self::RAM),
            "SWAP" => Ok(Self::SWAP),
            "COMB" => Ok(Self::COMB),
            _ => Err(std::io::ErrorKind::NotFound),
        }
    }
}

fn main() {
    let uid = users::get_current_uid();
    let user = users::get_user_by_uid(uid).unwrap_or_else(|| {
        eprintln!("FATAL: could not retrieve user from current uid ({})", uid);
        std::process::exit(1)
    });
    let shell = user.shell();
    let meminfo =
        fs::read_to_string("/proc/meminfo").expect("/proc/meminfo should exist and be readable");
    let mem_total = find_i64_in_line_starting_with(&meminfo, "MemTotal:").unwrap_or(0);
    let mem_available = find_i64_in_line_starting_with(&meminfo, "MemAvailable:").unwrap_or(0);
    let swap_total = find_i64_in_line_starting_with(&meminfo, "SwapTotal:").unwrap_or(0);
    let swap_free = find_i64_in_line_starting_with(&meminfo, "SwapFree:").unwrap_or(0);
    let ram_ratio = if mem_total == 0 {
        0f64
    } else {
        (mem_total - mem_available) as f64 / (mem_total) as f64
    };
    let swap_ratio = if swap_total == 0 {
        0f64
    } else {
        (swap_total - swap_free) as f64 / (swap_total) as f64
    };
    let comb_ratio = if (mem_total + swap_total) == 0 {
        0f64
    } else {
        ((mem_total + swap_total) - (mem_available + swap_free)) as f64
            / (mem_total + swap_total) as f64
    };
    let args = env::args().collect::<Vec<String>>();
    if argvec_asks_for_help() || args.len() != 4 {
        eprintln!("Usage:");
        eprintln!("  {} <RAM|SWAP|COMB> <percentage> <command>", args[0]);
        eprintln!("    Example:");
        eprintln!("      {} SWAP 80 reboot", args[0]);
        eprintln!("        will issue \"reboot\" command if over 80% of SWAP usage");
        eprintln!("    Example:");
        eprintln!("      {} RAM 60 true", args[0]);
        eprintln!("        will return fail state for a shell script if over 60% of RAM usage");
        eprintln!("    Current status:");
        eprintln!("      RAM:      {:6.2}%", ram_ratio * 100f64);
        eprintln!("      SWAP:     {:6.2}%", swap_ratio * 100f64);
        eprintln!("      Combined: {:6.2}%", comb_ratio * 100f64);
        std::process::exit(1)
    }
    let comp_mode = ComparationMode::try_from(args[1].as_str()).unwrap_or_else(|_| {
        eprintln!(
            "FATAL: The comparation should be \"RAM\", \"SWAP\" or \"COMB\", not {:?}",
            args[1]
        );
        std::process::exit(1)
    });
    let max_acceptable_ratio = args[2]
        .parse::<f64>()
        .and_then(|x| {
            if x > 0f64 && x < 100f64 {
                Ok(x)
            } else {
                "x".parse()
            }
        })
        .unwrap_or_else(|_| {
            eprintln!(
                "FATAL: The usage ratio should a decimal larger than zero and smaller than 100, not {:?}",
                args[2]
            );
            std::process::exit(1)
        })
        / 100f64;
    let measured_ratio = match comp_mode {
        ComparationMode::RAM => ram_ratio,
        ComparationMode::SWAP => swap_ratio,
        ComparationMode::COMB => comb_ratio,
    };
    if measured_ratio >= max_acceptable_ratio {
        std::process::exit(
            std::process::Command::new(shell)
                .arg("-c")
                .arg(&args[3])
                .spawn()
                .unwrap_or_else(|e| {
                    eprintln!("FATAL: while spawning shell \"sh\": {}", e);
                    std::process::exit(1)
                })
                .wait()
                .unwrap_or_else(|e| {
                    eprintln!("FATAL: while waiting for process: {}", e);
                    std::process::exit(1)
                })
                .code()
                .unwrap_or_else(|| {
                    eprintln!("FATAL: no value found while retrieving return code");
                    std::process::exit(1)
                }),
        )
    }
}
