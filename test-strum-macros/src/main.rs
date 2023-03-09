use std::str::FromStr;
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, Display, EnumIter};

fn main() {
    run();
}

#[derive(Display, AsRefStr, EnumIter)]
#[allow(unused)]
enum AiceServices {
    Giganto,
    Hog,
    Ntp,
    Piglet,
    REconverge,
    REview,
    Ssh,
    Syslog,
}

impl FromStr for AiceServices {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "giganto" => Ok(AiceServices::Giganto),
            "hog" => Ok(AiceServices::Hog),
            "ntp" => Ok(AiceServices::Ntp),
            "piglet" => Ok(AiceServices::Piglet),
            "reconverge" => Ok(AiceServices::REconverge),
            "review" => Ok(AiceServices::REview),
            "ssh" => Ok(AiceServices::Ssh),
            "syslog" => Ok(AiceServices::Syslog),
            _ => Err(()),
        }
    }
}

fn run() {
    let giganto = AiceServices::Giganto.to_string();
    let ssh = AiceServices::Ssh.to_string();
    let syslog: &'static str = AiceServices::Syslog.as_ref();
    let reconverge: &'static str = AiceServices::REconverge.as_ref();
    println!("giganto.to_string()={giganto}");
    println!("SSH.into()={ssh}");
    println!("syslog.afref()={syslog}");
    println!("reconverge.afref()={reconverge}");

    println!("Enum iteration:");
    for service in AiceServices::iter() {
        println!("{service}");
    }

    let services = vec!["giganto", "reconverge", "syslog", "tivan"];
    println!("Enum from str");
    for service in services {
        match AiceServices::from_str(service) {
            Ok(s) => println!("service {service} = {s}"),
            Err(e) => eprintln!("Failed to parse {service}. {e:?}"),
        }
    }
}
