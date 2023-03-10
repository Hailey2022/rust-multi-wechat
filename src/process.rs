use crate::utils::*;
use crate::winapi::*;

use std::io;
use std::mem;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Process {
    pid: u32,
    name: String,
    handle: HANDLE,
}

impl Process {
    pub fn pid(&self) -> u32 {
        self.pid
    }
}

impl Process {
    pub fn new(handle: HANDLE, pid: u32, name: &str) -> Self {
        Self {
            pid: pid,
            name: name.into(),
            handle,
        }
    }

    pub fn from_pid(pid: u32) -> Option<Self> {
        let handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS, FALSE, pid) };
        if handle.is_null() {
            return None;
        }
        let name = get_process_name(handle);
        Some(Self::new(handle, pid, name.as_str()))
    }

    pub fn from_pid_and_name(pid: u32, name: &str) -> Option<Self> {
        let handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS, FALSE, pid) };
        if handle.is_null() {
            return None;
        }

        Some(Self::new(handle, pid, name))
    }

    pub fn find_first_by_name(name: &str) -> Option<Self> {
        match find_process_by_name(name).unwrap_or_default().first() {
            Some(v) => Process::from_pid(v.pid),
            None => None,
        }
    }

    pub fn close(&self) -> io::Result<()> {
        if self.handle.is_null() {
            return Ok(());
        }
        let result = unsafe { CloseHandle(self.handle) };
        if result != 0 {
            return Ok(());
        }
        Err(get_last_error())
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

pub fn get_process_name(handle: HANDLE) -> String {
    let mut buf = [0u16; MAX_PATH + 1];
    unsafe {
        GetModuleBaseNameW(handle, 0 as _, buf.as_mut_ptr(), MAX_PATH as DWORD + 1);
        return wchar_to_string(&buf);
    };
}

pub fn find_process_by_name(name: &str) -> Result<Vec<Process>, io::Error> {
    let handle = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0 as _) };

    if handle.is_null() || handle == INVALID_HANDLE_VALUE {
        return Err(get_last_error());
    }

    let mut result: Vec<Process> = Vec::new();
    let mut _name: String;

    let mut entry: PROCESSENTRY32 = unsafe { ::std::mem::zeroed() };
    entry.dwSize = mem::size_of::<PROCESSENTRY32>() as u32;

    while 0 != unsafe { Process32Next(handle, &mut entry) } {
        _name = char_to_string(&entry.szExeFile);
        entry.szExeFile = unsafe { ::std::mem::zeroed() };
        if name.len() > 0 && !_name.contains(name) {
            continue;
        }

        match Process::from_pid_and_name(entry.th32ProcessID, _name.as_str()) {
            Some(v) => result.push(v),
            None => {}
        }
    }

    result.reverse();
    Ok(result)
}
