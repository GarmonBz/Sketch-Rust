#[cfg(windows)]
fn main() {
    use std::{env, fs, path::PathBuf, process::Command};
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let icon = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/app-icon.ico");
    println!("cargo:rerun-if-changed={}", icon.display());
    println!("cargo:rerun-if-env-changed=RC");
    let rc = env::var_os("RC")
        .map(PathBuf::from)
        .filter(|p| p.is_file())
        .or_else(|| {
            let kits = PathBuf::from(env::var_os("ProgramFiles(x86)")?).join("Windows Kits/10/bin");
            let host = if cfg!(target_arch = "aarch64") {
                "arm64"
            } else if cfg!(target_arch = "x86_64") {
                "x64"
            } else {
                "x86"
            };
            let mut versions: Vec<_> = fs::read_dir(&kits)
                .ok()?
                .filter_map(Result::ok)
                .map(|e| e.path())
                .collect();
            versions.sort();
            versions.reverse();
            versions
                .into_iter()
                .map(|p| p.join(host).join("rc.exe"))
                .find(|p| p.is_file())
                .or_else(|| {
                    let p = kits.join(host).join("rc.exe");
                    p.is_file().then_some(p)
                })
        })
        .expect("Windows SDK rc.exe is required; install the Windows SDK or set RC");
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    let resource = out.join("app-icon.rc");
    let compiled = out.join("app-icon.res");
    fs::write(
        &resource,
        format!(
            "1 ICON \"{}\"\n",
            icon.display().to_string().replace('\\', "/")
        ),
    )
    .expect("write icon resource");
    let status = Command::new(rc)
        .arg("/nologo")
        .arg("/fo")
        .arg(&compiled)
        .arg(&resource)
        .status()
        .expect("run Windows resource compiler");
    assert!(status.success(), "Windows icon resource compilation failed");
    println!("cargo:rustc-link-arg-bin=sketch-app={}", compiled.display());
}
#[cfg(not(windows))]
fn main() {}
