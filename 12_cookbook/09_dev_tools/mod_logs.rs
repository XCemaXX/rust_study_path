mod first_level {
    mod second_level {
        pub fn log() {
            log::warn!("from 2");
            log::info!("from 2");
            log::debug!("from 2");
        }
    }
    pub fn log() {
        log::warn!("from 1");
        log::info!("from 1");
        log::debug!("from 1");
        second_level::log();
    }
}

// INSTEAD_OF_RUST_LOG="warn,cook_mod_logs::first_level=info,cook_mod_logs::first_level::second_level=debug" cargo run --bin cook_conf_logs
fn main() {
    env_logger::Builder::from_env("INSTEAD_OF_RUST_LOG").init();

    log::warn!("from root");
    log::info!("from root");
    log::debug!("from root");
    first_level::log();
}
