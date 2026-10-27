use std::process::Command;

/// 让 hugo 子进程与图形应用解耦：
/// - 独立进程组，退出时不会连带影响应用；
/// - Linux 下设置 PDEATHSIG，应用崩溃或被 kill 时内核会给子进程发 SIGTERM，
///   避免 `tauri dev` 重启后留下孤儿 `hugo server` 占用端口。
#[cfg(unix)]
pub fn isolate(command: &mut Command) {
    use std::os::unix::process::CommandExt;

    command.process_group(0);

    #[cfg(target_os = "linux")]
    unsafe {
        command.pre_exec(|| {
            libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM);
            Ok(())
        });
    }
}

#[cfg(not(unix))]
pub fn isolate(_command: &mut Command) {}
