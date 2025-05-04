use std::fmt::Debug;

use spdlog::{error, warn};

pub fn log_error<E>(e: &E)
where
    E: Debug,
{
    error!("{e:?}");
}

pub fn log_warn<E>(e: &E)
where
    E: Debug,
{
    warn!("{e:?}");
}
