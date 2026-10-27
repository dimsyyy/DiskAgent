use std::ffi::{c_void, OsStr};
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::ptr;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::model::VolumeInfo;

type Handle = *mut c_void;
type Bool = i32;

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct FileTime {
    low: u32,
    high: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct WinSystemTime {
    year: u16,
    month: u16,
    day_of_week: u16,
    day: u16,
    hour: u16,
    minute: u16,
    second: u16,
    milliseconds: u16,
}

#[repr(C)]
struct TokenElevation {
    token_is_elevated: u32,
}

#[link(name = "kernel32")]
extern "system" {
    fn GetDiskFreeSpaceExW(
        directory: *const u16,
        free_to_caller: *mut u64,
        total: *mut u64,
        total_free: *mut u64,
    ) -> Bool;
    fn GetDiskFreeSpaceW(
        root: *const u16,
        sectors_per_cluster: *mut u32,
        bytes_per_sector: *mut u32,
        free_clusters: *mut u32,
        total_clusters: *mut u32,
    ) -> Bool;
    fn GetVolumeInformationW(
        root: *const u16,
        volume_name: *mut u16,
        volume_name_len: u32,
        serial: *mut u32,
        max_component: *mut u32,
        flags: *mut u32,
        fs_name: *mut u16,
        fs_name_len: u32,
    ) -> Bool;
    fn GetCompressedFileSizeW(path: *const u16, high: *mut u32) -> u32;
    fn GetLastError() -> u32;
    fn FileTimeToLocalFileTime(input: *const FileTime, output: *mut FileTime) -> Bool;
    fn FileTimeToSystemTime(input: *const FileTime, output: *mut WinSystemTime) -> Bool;
    fn GetCurrentProcess() -> Handle;
    fn CloseHandle(handle: Handle) -> Bool;
}

#[link(name = "advapi32")]
extern "system" {
    fn OpenProcessToken(process: Handle, access: u32, token: *mut Handle) -> Bool;
    fn GetTokenInformation(
        token: Handle,
        class: i32,
        info: *mut c_void,
        info_len: u32,
        return_len: *mut u32,
    ) -> Bool;
}

#[link(name = "shell32")]
extern "system" {
    fn ShellExecuteW(
        window: Handle,
        operation: *const u16,
        file: *const u16,
        parameters: *const u16,
        directory: *const u16,
        show: i32,
    ) -> isize;
}

fn wide(value: &OsStr) -> Vec<u16> {
    value.encode_wide().chain(std::iter::once(0)).collect()
}

fn wide_str(value: &str) -> Vec<u16> {
    OsStr::new(value)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn volume_root(path: &Path) -> Vec<u16> {
    let text = path.to_string_lossy();
    let root = if text.len() >= 3 && text.as_bytes()[1] == b':' {
        format!("{}:\\", &text[..1])
    } else {
        text.into_owned()
    };
    wide_str(&root)
}

pub fn volume_info(path: &Path) -> VolumeInfo {
    let mut info = VolumeInfo {
        cluster_size: 4096,
        fs_name: String::new(),
        ..Default::default()
    };

    let path_wide = wide(path.as_os_str());
    let mut free_to_caller: u64 = 0;
    let mut total: u64 = 0;
    let mut total_free: u64 = 0;
    let ok = unsafe {
        GetDiskFreeSpaceExW(
            path_wide.as_ptr(),
            &mut free_to_caller,
            &mut total,
            &mut total_free,
        )
    };
    if ok != 0 {
        info.free_bytes = total_free;
        info.total_bytes = total;
    }

    let root = wide_str(&path.to_string_lossy());
    let root_wide = volume_root(path);
    let _ = root;

    let mut spc: u32 = 0;
    let mut bps: u32 = 0;
    let mut free_clusters: u32 = 0;
    let mut total_clusters: u32 = 0;
    let ok = unsafe {
        GetDiskFreeSpaceW(
            root_wide.as_ptr(),
            &mut spc,
            &mut bps,
            &mut free_clusters,
            &mut total_clusters,
        )
    };
    if ok != 0 && spc > 0 && bps > 0 {
        info.cluster_size = (spc as u64) * (bps as u64);
    }

    let mut volume_name = [0u16; 64];
    let mut serial: u32 = 0;
    let mut max_component: u32 = 0;
    let mut flags: u32 = 0;
    let mut fs_name = [0u16; 32];
    let ok = unsafe {
        GetVolumeInformationW(
            root_wide.as_ptr(),
            volume_name.as_mut_ptr(),
            volume_name.len() as u32,
            &mut serial,
            &mut max_component,
            &mut flags,
            fs_name.as_mut_ptr(),
            fs_name.len() as u32,
        )
    };
    if ok != 0 {
        info.fs_name = String::from_utf16_lossy(&fs_name)
            .trim_end_matches('\0')
            .to_string();
    }

    info
}

pub fn compressed_size(path: &Path) -> Option<u64> {
    let wide_path = wide(path.as_os_str());
    let mut high: u32 = 0;
    let low = unsafe { GetCompressedFileSizeW(wide_path.as_ptr(), &mut high) };
    if low == u32::MAX {
        let err = unsafe { GetLastError() };
        if err != 0 {
            return None;
        }
    }
    Some(((high as u64) << 32) | low as u64)
}

pub fn open_location(path: &Path, select: bool) {
    let mut cmd = std::process::Command::new("explorer.exe");
    if select {
        cmd.arg(format!("/select,{}", path.display()));
    } else {
        cmd.arg(path);
    }
    let _ = cmd.spawn();
}

/// Opens a URL in the user's default browser.
pub fn open_url(url: &str) -> bool {
    let operation = wide_str("open");
    let file = wide_str(url);
    let result = unsafe {
        ShellExecuteW(
            ptr::null_mut(),
            operation.as_ptr(),
            file.as_ptr(),
            ptr::null(),
            ptr::null(),
            1,
        )
    };
    result as usize > 32
}

/// Permanently deletes a file or directory (bypassing the Recycle Bin).
/// Read-only attributes are cleared first, and reparse points are removed
/// without following the link target.
pub fn delete_permanently(path: &Path) -> std::io::Result<()> {
    remove_path(path)
}

fn remove_path(path: &Path) -> std::io::Result<()> {
    use std::os::windows::fs::MetadataExt;

    let metadata = std::fs::symlink_metadata(path)?;

    // Junctions and symlinks: delete the link itself, never recurse into it.
    if metadata.file_attributes() & 0x400 != 0 {
        return match std::fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(file_err) => std::fs::remove_dir(path).map_err(|_| file_err),
        };
    }

    if metadata.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            remove_path(&entry.path())?;
        }
        clear_readonly(path, &metadata);
        std::fs::remove_dir(path)
    } else {
        clear_readonly(path, &metadata);
        std::fs::remove_file(path)
    }
}

fn clear_readonly(path: &Path, metadata: &std::fs::Metadata) {
    if metadata.permissions().readonly() {
        let mut permissions = metadata.permissions();
        permissions.set_readonly(false);
        let _ = std::fs::set_permissions(path, permissions);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permanent_delete_removes_readonly_tree() {
        let base = std::env::temp_dir().join(format!(
            "diskagent-delete-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("sub")).expect("create test directory");
        std::fs::write(base.join("sub").join("a.txt"), b"abc").expect("write nested file");
        let readonly = base.join("readonly.txt");
        std::fs::write(&readonly, b"x").expect("write readonly file");
        let mut permissions = std::fs::metadata(&readonly)
            .expect("readonly metadata")
            .permissions();
        permissions.set_readonly(true);
        std::fs::set_permissions(&readonly, permissions).expect("set readonly");

        delete_permanently(&base).expect("delete tree");
        assert!(!base.exists());
    }
}

pub fn is_elevated() -> bool {
    unsafe {
        let mut token: Handle = ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), 0x0008, &mut token) == 0 {
            return false;
        }
        let mut info = TokenElevation {
            token_is_elevated: 0,
        };
        let mut returned: u32 = 0;
        let ok = GetTokenInformation(
            token,
            20,
            &mut info as *mut _ as *mut c_void,
            std::mem::size_of::<TokenElevation>() as u32,
            &mut returned,
        );
        CloseHandle(token);
        ok != 0 && info.token_is_elevated != 0
    }
}

pub fn run_as_admin() -> bool {
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    let verb = wide_str("runas");
    let file = wide(exe.as_os_str());
    let result = unsafe {
        ShellExecuteW(
            ptr::null_mut(),
            verb.as_ptr(),
            file.as_ptr(),
            ptr::null(),
            ptr::null(),
            1,
        )
    };
    result as usize > 32
}

fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Formats a file timestamp as `dd.MM.yyyy` in local time.
pub fn format_time(time: SystemTime) -> String {
    let Ok(duration) = time.duration_since(UNIX_EPOCH) else {
        return "—".to_string();
    };

    let ticks = (duration.as_secs() + 11_644_473_600) as u128 * 10_000_000
        + (duration.subsec_nanos() as u128 / 100);
    let filetime = FileTime {
        low: (ticks & 0xFFFF_FFFF) as u32,
        high: ((ticks >> 32) & 0xFFFF_FFFF) as u32,
    };

    let mut local = FileTime::default();
    let mut st = WinSystemTime::default();
    let ok = unsafe { FileTimeToLocalFileTime(&filetime, &mut local) };
    if ok != 0 {
        let ok = unsafe { FileTimeToSystemTime(&local, &mut st) };
        if ok != 0 {
            return format!("{:02}.{:02}.{:04}", st.day, st.month, st.year);
        }
    }

    let days = (duration.as_secs() / 86_400) as i64;
    let (year, month, day) = civil_from_days(days);
    format!("{:02}.{:02}.{:04}", day, month, year)
}
