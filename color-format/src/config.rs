use ::core::sync::atomic::{AtomicBool, Ordering};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref CONFIG: Config = Config::find_out();
}

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
                && atty::is(atty::Stream::Stdout)
                && atty::is(atty::Stream::Stderr)
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

pub fn set_override(colorize: bool) {
    CONFIG.override_colorized.store(colorize, Ordering::Relaxed);
    CONFIG.override_set.store(true, Ordering::Relaxed);
}
pub fn unset_override() {
    CONFIG.override_set.store(false, Ordering::Relaxed);
}

fn env_set(name: &str) -> bool {
    let res = std::env::var(name);
    match res.as_ref().map(|s| s.as_str()) {
        Ok("0") | Err(_) => false,
        _ => true
    }
}
