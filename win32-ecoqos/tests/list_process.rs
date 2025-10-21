use win32_ecoqos::utils::Processes;

#[test]
fn test_csrss_process_name() -> windows_result::Result<()> {
    let procs = Processes::try_new()?;
    let processes: Vec<_> = procs.collect();

    assert!(processes.len() > 1);

    Ok(())
}
