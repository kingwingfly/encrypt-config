use snafu::Snafu;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub(crate)), context(suffix(false)))]
pub enum ConfigError {}

pub type ConfigResult<T> = Result<T, ConfigError>;
