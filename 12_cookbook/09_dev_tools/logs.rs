#[cfg(feature = "to_syslog")]
use syslog::Facility;

fn execute_query(query: &str) {
    log::debug!("Executing query: {}", query);
}

fn make_error() -> Result<(), &'static str> {
    Err("Unpredictable error")
}

#[cfg(feature = "custom")]
mod custom_logger {
    use log::{Level, LevelFilter, Metadata, Record};
    pub static CONSOLE_LOGGER: ConsoleLogger = ConsoleLogger;

    pub struct ConsoleLogger;

    impl log::Log for ConsoleLogger {
        fn enabled(&self, metadata: &Metadata) -> bool {
            metadata.level() <= Level::Info
        }

        fn log(&self, record: &Record) {
            if self.enabled(record.metadata()) {
                println!("CustmoLogger: {} - {}", record.level(), record.args())
            }
        }

        fn flush(&self) {}
    }

    pub fn set_level_info() {
        log::set_max_level(LevelFilter::Info);
    }
}

fn main() {
    #[cfg(feature = "simple")]
    env_logger::init();
    #[cfg(feature = "to_stdout")]
    env_logger::Builder::new()
        .target(env_logger::Target::Stdout)
        .init();
    #[cfg(feature = "custom")]
    {
        log::set_logger(&custom_logger::CONSOLE_LOGGER).unwrap();
        custom_logger::set_level_info();
    }
    #[cfg(feature = "to_syslog")]
    syslog::init(
        Facility::LOG_USER,
        log::LevelFilter::Debug,
        Some("cook_logs"),
    )
    .unwrap();

    #[cfg(feature = "timestamps")]
    {
        use std::io::Write;
        env_logger::Builder::new()
            .format(|buf, record| {
                writeln!(
                    buf,
                    "{} [{}] - {}",
                    chrono::Local::now().format("%d.%m.%Y %H:%M:%S"),
                    record.level(),
                    record.args()
                )
            })
            .filter(None, log::LevelFilter::Info)
            .init();
    }
    #[cfg(feature = "to_file")]
    {
        use log::LevelFilter;
        use log4rs::append::file::FileAppender;
        use log4rs::config::{Appender, Config, Root};
        use log4rs::encode::pattern::PatternEncoder;

        let logfile = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("[{d}] {l} - {m}\n")))
            .build("12_cookbook/09_dev_tools/log_to_file.txt")
            .unwrap();

        let config = Config::builder()
            .appender(Appender::builder().build("logfile", Box::new(logfile)))
            .build(Root::builder().appender("logfile").build(LevelFilter::Info))
            .unwrap();
        log4rs::init_config(config).unwrap();
    }

    execute_query("DROP TABLE micro");
    if let Err(err) = make_error() {
        log::error!("Error! {}", err);
    }
    log::info!("hello log");
    log::warn!("warning");
    log::error!("oops");
}
