use std::collections::HashSet;
use std::error::Error;

#[cfg(target_os = "linux")]
pub async fn get_running_processes() -> Result<HashSet<String>, Box<dyn Error + Send + Sync>> {
    let mut pids = tokio::fs::read_dir("/proc")
        .await
        .map_err(|e| format!("error reading /proc -> {e}"))?;

    let mut process_list = HashSet::with_capacity(300);

    while let Ok(Some(dir)) = pids.next_entry().await {
        if let Ok(name) = tokio::fs::read_link(dir.path().join("exe")).await {
            if let Some(name) = name.file_name() {
                process_list.insert(name.to_string_lossy().to_string());
            }
        }
    }

    Ok(process_list)
}

#[cfg(target_os = "macos")]
pub async fn get_running_processes() -> Result<HashSet<String>, ProcessError> {
    compile_error!("macos not yet implemented");
    todo!()
}

#[cfg(target_os = "windows")]
fn from_utf16_until_nul_byte(input: &[u16]) -> Result<String, Box<dyn Error + Send + Sync>> {
    let mut split = input.split(|char| *char == 0);
    let bytes = split
        .next()
        .ok_or_else(|| "no nul byte found".to_string())?;
    Ok(String::from_utf16(bytes)?)
}

#[cfg(target_os = "windows")]
pub async fn get_running_processes() -> Result<HashSet<String>, Box<dyn Error + Send + Sync>> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    use windows_sys::Win32::System::Kernel::NULL64;

    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        cntUsage: 0,
        th32ProcessID: 0,
        th32DefaultHeapID: 0,
        th32ModuleID: 0,
        cntThreads: 0,
        th32ParentProcessID: 0,
        pcPriClassBase: 0,
        dwFlags: 0,
        szExeFile: [0; 260],
    };

    let mut process_list = HashSet::with_capacity(300);

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, NULL64);

        if Process32FirstW(snapshot, &mut entry) == 0 {
            CloseHandle(snapshot);
            return Err("Process32First failed".into());
        }
        process_list.insert(from_utf16_until_nul_byte(&entry.szExeFile)?);

        while Process32NextW(snapshot, &mut entry) != 0 {
            process_list.insert(from_utf16_until_nul_byte(&entry.szExeFile)?);
        }

        Ok(process_list)
    }
}
