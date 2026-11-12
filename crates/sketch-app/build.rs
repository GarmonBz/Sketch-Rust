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
    compile_apex_component();
}
#[cfg(not(windows))]
fn main() {}

#[cfg(windows)]
fn compile_apex_component() {
    use std::{env, fs, path::PathBuf, process::Command};
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/apex_compat.cs");
    let legacy = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/apex_legacy.cs");
    println!("cargo:rerun-if-changed={}", legacy.display());
    let geometry = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/apex_geometry.cs");
    println!("cargo:rerun-if-changed={}", geometry.display());
    let drafts = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/apex_drafts.cs");
    println!("cargo:rerun-if-changed={}", drafts.display());
    let codes = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/apex_codes.cs");
    println!("cargo:rerun-if-changed={}", codes.display());
    let codec = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/apex_file.cs");
    println!("cargo:rerun-if-changed={}", source.display());
    println!("cargo:rerun-if-changed={}", codec.display());
    println!("cargo:rerun-if-env-changed=APEX_INTEGRATION_SDK");
    let system = PathBuf::from(env::var_os("SystemRoot").expect("SystemRoot"));
    let sdk = env::var_os("APEX_INTEGRATION_SDK")
        .map(PathBuf::from)
        .or_else(|| {
            let gac = system.join("assembly/GAC_MSIL/Apex.Integration.DotNet");
            let mut versions: Vec<_> = fs::read_dir(gac)
                .ok()?
                .filter_map(Result::ok)
                .map(|e| e.path())
                .collect();
            versions.sort();
            versions.reverse();
            versions
                .into_iter()
                .map(|p| p.join("Apex.Integration.DotNet.dll"))
                .find(|p| p.is_file())
        })
        .expect("Set APEX_INTEGRATION_SDK to the installed Apex.Integration.DotNet.dll");
    println!("cargo:rerun-if-changed={}", sdk.display());
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR")).join("apex-compat.dll");
    let status = Command::new(system.join("Microsoft.NET/Framework64/v4.0.30319/csc.exe"))
        .args([
            "/nologo",
            "/target:winexe",
            "/optimize+",
            "/reference:System.Windows.Forms.dll",
            "/reference:System.Drawing.dll",
            "/reference:System.Web.Extensions.dll",
        ])
        .arg(format!("/out:{}", output.display()))
        .arg(format!("/reference:{}", sdk.display()))
        .arg(format!(
            "/resource:{},SketchRust.HostContract",
            sdk.display()
        ))
        .arg(source.to_string_lossy().replace('/', "\\"))
        .arg(geometry.to_string_lossy().replace('/', "\\"))
        .arg(drafts.to_string_lossy().replace('/', "\\"))
        .arg(codes.to_string_lossy().replace('/', "\\"))
        .arg(codec.to_string_lossy().replace('/', "\\"))
        .arg(legacy.to_string_lossy().replace('/', "\\"))
        .status()
        .expect("run .NET Framework component compiler");
    assert!(
        status.success(),
        "Compile embedded Apex compatibility component"
    );
    // ExecuteAssembly must have an entry point without allocating a console.
    let bytes = fs::read(&output).expect("read embedded host component");
    let pe = u32::from_le_bytes(bytes[0x3c..0x40].try_into().unwrap()) as usize;
    assert_eq!(
        u16::from_le_bytes(bytes[pe + 92..pe + 94].try_into().unwrap()),
        2,
        "The embedded host component must use the Windows GUI subsystem"
    );
}
