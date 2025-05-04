use std::fmt::Debug;

use spdlog::{error, warn};

pub fn log_error<'a, E>(e: &'a E)
where
    E: Debug,
{
    error!("{e:?}");
}

pub fn log_warn<'a, E>(e: &'a E)
where
    E: Debug,
{
    warn!("{e:?}");
}
