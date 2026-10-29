pub mod describe;
pub mod health;
pub mod tools;

use crate::backends::Backend;
use crate::config::Config;
use crate::formats::Format;
use crate::tools::registry::Registry;

pub struct AppState {
    pub config: Config,
    pub backend: Backend,
    pub registry: Registry,
    pub formats: Vec<Format>,
}
