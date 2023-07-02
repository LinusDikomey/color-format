use ::core::sync::atomic::{AtomicBool, Ordering};
use std::{io::IsTerminal, sync::OnceLock};

pub struct Config {
    colorize: bool,
    override_set: AtomicBool,
    override_colorized: AtomicBool,
}
impl Config {
    fn find_out() -> Self {
        let colorize =
            env_set("CLICOLOR_FORCE") | env_set("CLICOLOR") | (
                !env_set("NO_COLOR")
                && std::io::stdout().is_terminal()
                && std::io::stderr().is_terminal()
                //&& atty::is(atty::Stream::Stdout)
                //&& atty::is(atty::Stream::Stderr)
            );
        Self {
            colorize,
            override_set: AtomicBool::new(false),
            override_colorized: AtomicBool::new(false),
        }
    }
    pub fn colorize(&self) -> bool {
        if self.override_set.load(Ordering::Relaxed) {
            self.override_colorized.load(Ordering::Relaxed)
        } else {
            self.colorize
        }
    }
}

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn config() -> &'static Config {
    CONFIG.get_or_init(Config::find_out)
}

pub fn set_override(colorize: bool) {
    let config = config();
    config.override_colorized.store(colorize, Ordering::Relaxed);
    config.override_set.store(true, Ordering::Relaxed);
}
pub fn unset_override() {
    config().override_set.store(false, Ordering::Relaxed);
}

fn env_set(name: &str) -> bool {
    !matches!(std::env::var(name).as_ref().map(String::as_str), Ok("0") | Err(_))
}
