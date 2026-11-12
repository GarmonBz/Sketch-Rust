# Installing Sketch Rust

You don't need Rust or any programming tools. Sketch Rust is a normal Windows program.
You need a 64-bit Windows 10 or 11 PC.

## Install it (recommended)

1. Open the **[Releases page](https://github.com/cpursley/Sketch-Rust/releases)**. The newest
   version is at the top.
2. Under **Assets**, click the file ending in **`.msi`**, for example
   `SketchRust-0.1.29-x64.msi`. Your browser downloads it.
3. Open the downloaded file. Click it in the browser's downloads list, or double-click it
   in your **Downloads** folder.
4. Windows may warn you, because the program is not code-signed yet:
   - **Edge** says the file "isn't commonly downloaded": click **…** > **Keep** >
     **Show more** > **Keep anyway**.
   - **"Windows protected your PC"** (blue box): click **More info** > **Run anyway**.
5. In the installer, click **Next**, accept the licence, click **Next**, then **Install**,
   then **Finish**. You don't need an administrator password.
6. Sketch Rust opens. Next time, start it from the **Start menu** or the **Sketch Rust**
   shortcut on your desktop.

**Updating:** download and run the newer `.msi` the same way. It replaces the old version.
If you have edited the symbol library, keep a copy of `symbols.json` first, because
updating replaces it.

**Uninstalling:** go to **Settings > Apps > Installed apps**, find **Sketch Rust**, then
click **…** > **Uninstall**.

## Run it without installing (portable)

1. On the Releases page, download the file ending in **`windows-x64.zip`**.
2. Right-click the zip and choose **Extract All…**, then **Extract**. Don't run the program
   from inside the zip, because it needs the files next to it.
3. In the extracted folder, double-click **`sketch-app.exe`**. If the same Windows warning
   appears, use **More info** > **Run anyway**.

## Use it with Appraise-It Pro

1. Install Sketch Rust where it will stay.
2. In Sketch Rust, open **Help > Appraise-It Integration** and click **Enable
   integration**. Allow the Windows administrator prompt if one appears.
3. Restart Appraise-It Pro and use its usual Apex sketch button.

**Restore previous provider** in the same panel switches the report back to Apex. Setup
changes the shared Apex launch setting; it does not modify Appraise-It Pro. See
[APEX_REPLACEMENT.md](APEX_REPLACEMENT.md) for what is covered.

## For IT and advanced users

- Silent install: `msiexec /i SketchRust-<version>-x64.msi /qn`. It installs per-user to
  `%LOCALAPPDATA%\Programs\SketchRust`.
- Each download has a `.sha256` file to check it against.
