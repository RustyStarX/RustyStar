use std::{collections::HashMap, time::Duration};

use futures_lite::StreamExt;
use serde::Deserialize;
use wmi::{FilterValue, WMIConnection};

use crate::Error;

#[derive(Deserialize, Debug)]
#[serde(rename = "__InstanceCreationEvent")]
#[serde(rename_all = "PascalCase")]
struct NewProcessEvent {
    target_instance: Process,
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename = "Win32_Process")]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
pub struct Process {
    pub process_id: u32,
    pub parent_process_id: u32,
    pub name: String,
}

pub async fn listen_process_creation(mut callback: impl FnMut(Process)) -> Result<(), Error> {
    let wmi_con = WMIConnection::new()?;

    let mut filters = HashMap::<String, FilterValue>::new();
    filters.insert("TargetInstance".to_owned(), FilterValue::is_a::<Process>()?);

    let mut stream = wmi_con
        .async_filtered_notification::<NewProcessEvent>(&filters, Some(Duration::from_secs(1)))?;

    while let Some(result) = stream.next().await {
        let process = result?.target_instance;
        callback(process);
    }

    Ok(())
}
