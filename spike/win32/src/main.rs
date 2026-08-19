//! Throwaway spike: print the foreground application on Windows.
//!
//! The ARCHITECTURE §6 pipeline, nothing more:
//!   GetForegroundWindow → GetWindowThreadProcessId → OpenProcess
//!   → QueryFullProcessImageNameW → file stem
//!
//! Note what is *not* here and never will be: GetWindowTextW. There is no code
//! path in this spike that can observe a window title (AGENTS.md hard rule 2).
//!
//! Runs bounded (N polls, 1/sec) so CI can execute it without hanging.

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;

use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, MAX_PATH};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
    PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

/// What one poll saw. `None` means "nothing in the foreground" — a real state on
/// a locked desktop or a headless CI session, and the backend must handle it
/// as cleanly as any other.
struct Foreground {
    pid: u32,
    exe_path: PathBuf,
}

/// pid → executable path. The half of the pipeline that does not depend on
/// there being an interactive desktop, so it can be verified anywhere.
fn exe_for_pid(pid: u32) -> Option<PathBuf> {
    unsafe {
        // LIMITED_INFORMATION is the least privilege that still allows
        // QueryFullProcessImageNameW, and it works against elevated processes
        // where PROCESS_QUERY_INFORMATION would be denied.
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false.into(), pid).ok()?;

        let mut buf = [0u16; MAX_PATH as usize];
        let mut len = buf.len() as u32;
        let result = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buf.as_mut_ptr()),
            &mut len,
        );
        let _ = CloseHandle(handle);
        result.ok()?;

        Some(PathBuf::from(OsString::from_wide(&buf[..len as usize])))
    }
}

fn foreground() -> Option<Foreground> {
    let pid = unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }
        let mut pid: u32 = 0;
        // Returns the thread id; we only want the process id it writes out.
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        pid
    };
    if pid == 0 {
        return None;
    }
    Some(Foreground { pid, exe_path: exe_for_pid(pid)? })
}

/// Prove OpenProcess + QueryFullProcessImageNameW work on this machine by
/// resolving our own pid, whose answer we can check. Without this, a headless
/// run that finds no foreground window would tell us nothing about whether the
/// resolution half of the pipeline works at all.
fn self_check() -> bool {
    let pid = unsafe { GetCurrentProcessId() };
    match exe_for_pid(pid) {
        Some(path) => {
            let stem = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            let ok = stem == "spike-win32";
            println!(
                "self-check: pid={pid} → exe_name={stem:?} path={:?} [{}]",
                path.display().to_string(),
                if ok { "PASS" } else { "UNEXPECTED NAME" }
            );
            ok
        }
        None => {
            println!("self-check: FAILED — could not resolve our own pid to an exe path");
            false
        }
    }
}

fn main() {
    let polls: usize = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(10);

    println!("spike: polling foreground app {polls}x at 1/sec (no window titles are read)");
    let resolution_works = self_check();

    let mut saw_foreground = false;
    for i in 0..polls {
        match foreground() {
            Some(fg) => {
                saw_foreground = true;
                // This stem is exactly what normalize_exe_name() consumes.
                let stem = fg
                    .exe_path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                println!(
                    "[{i:>2}] exe_name={stem:?} pid={} exe_path={:?}",
                    fg.pid,
                    fg.exe_path.display().to_string()
                );
            }
            None => println!("[{i:>2}] no foreground window (reports Activity::None)"),
        }
        if i + 1 < polls {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }

    println!("---");
    println!("pid → exe resolution: {}", if resolution_works { "WORKS" } else { "BROKEN" });
    println!(
        "foreground window seen: {}",
        if saw_foreground {
            "yes — full pipeline verified"
        } else {
            "no — headless/service session; needs an interactive desktop to confirm"
        }
    );
    // The resolution half is what a headless run can actually prove; fail the
    // run only on that, so CI stays meaningful without an interactive desktop.
    if !resolution_works {
        std::process::exit(1);
    }
    println!("spike: done");
}
