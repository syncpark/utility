mod call_roxy;
mod ufw;

use std::env;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let roxy = args.get(2) == Some(&"roxy".to_string());
    if let Some(cmd) = args.get(1) {
        match cmd.as_str() {
            "is-active" => {
                let ret = if roxy {
                    call_roxy::is_active()
                } else {
                    ufw::is_active()
                };
                match ret {
                    Ok(s) => println!("{s}"),
                    Err(e) => eprintln!("{e}"),
                }
            }
            "status" => {
                let ret = if roxy {
                    call_roxy::status()
                } else {
                    ufw::status()
                };

                match ret {
                    Ok(Some(rules)) => println!("{rules}"),
                    Ok(None) => println!("No rules"),
                    Err(e) => eprintln!("{e}"),
                }
            }
            "start" => {
                let ret = if roxy {
                    call_roxy::enable()
                } else {
                    ufw::enable()
                };
                match ret {
                    Ok(r) => println!("{r}"),
                    Err(e) => eprintln!("{e}"),
                }
            }
            "stop" => {
                let ret = if roxy {
                    call_roxy::disable()
                } else {
                    ufw::disable()
                };
                match ret {
                    Ok(r) => println!("{r}"),
                    Err(e) => eprintln!("{e}"),
                }
            }
            "allow" | "delete" | "deny" => {
                if let Some(params) = args.get(1..) {
                    ufw::update(params);
                }
            }
            _ => eprintln!("unknown command"),
        }
    }
}
