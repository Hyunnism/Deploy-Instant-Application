// core/exec.rs
// Satu-satunya pintu eksekusi proot (PRD Bagian 8.3)
// Seluruh pemanggilan ke sistem Linux di dalam rootfs WAJIB melalui fungsi ini.

use std::path::Path;
use tokio::process::Command;

/// Membangun Command untuk mengeksekusi perintah di dalam proot rootfs
pub fn proot_exec(
    proot_bin: &Path,
    rootfs_dir: &Path,
    tmp_dir: &Path,
    cwd: &str,
    cmd: &str,
    env_vars: &[(String, String)],
    no_seccomp: bool,
) -> Command {
    let mut c = Command::new(proot_bin);

    let shell_cmd = format!(
        "export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:$PATH; export HOME=/root; export LANG=C.UTF-8; {cmd}"
    );

    c.args([
        "-0",
        "-r",
        rootfs_dir.to_str().unwrap_or("/"),
        "-b",
        "/dev",
        "-b",
        "/proc",
        "-b",
        "/sys",
        "-w",
        cwd,
        "--kill-on-exit",
        "/bin/sh",
        "-c",
        &shell_cmd,
    ]);

    c.env("PROOT_TMP_DIR", tmp_dir);
    c.env("HOME", "/root");
    c.env("LANG", "C.UTF-8");
    c.env(
        "PATH",
        "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
    );

    // ponytail: PROOT_NO_SECCOMP=1 hanya bila seccomp bermasalah di device tertentu; lebih lambat.
    if no_seccomp {
        c.env("PROOT_NO_SECCOMP", "1");
    }

    for (k, v) in env_vars {
        c.env(k, v);
    }

    #[cfg(unix)]
    c.process_group(0);

    c.kill_on_drop(true);
    c
}
