use win32_ecoqos::utils::{Process, Processes};

fn main() -> windows_result::Result<()> {
    let processes: Vec<_> = Processes::try_new()?.collect();

    for Process {
        process_id,
        process_name,
        ..
    } in &processes
    {
        println!("{process_id:6}: {process_name:?}");
    }

    println!("found {} processes", processes.len());
    Ok(())
}
