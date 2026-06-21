mod config;
mod rain;
mod settings;

fn main() {
    let mut config = config::Config::load();

    if std::env::args().any(|a| a == "--settings") {
        match settings::run_settings(config) {
            Ok(new_config) => {
                config = new_config;
                if let Err(e) = config.save() {
                    eprintln!("Failed to save settings: {e}");
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("Settings error: {e}");
                std::process::exit(1);
            }
        }
    }

    if let Err(e) = rain::run_rain(&config) {
        eprintln!("Rain error: {e}");
        std::process::exit(1);
    }
}
