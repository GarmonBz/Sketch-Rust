# tools/shot.ps1 — capture the running sketch-app.exe window to a PNG.
# Usage: powershell -NoProfile -ExecutionPolicy Bypass -File tools\shot.ps1 [-Out path.png] [-Width 1400] [-Height 860]
# Resizes the window to the reference size (1400x860 client-ish) so captures are
# directly comparable with refs/ui-spec/gallery/*.png (captured at 1400x860).
param(
  [string]$Out = "$PSScriptRoot\..\target\shot.png",
  [int]$Width = 1400,
  [int]$Height = 860,
  [switch]$NoResize   # capture at the window's current size (real-DPI review)
)
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System; using System.Runtime.InteropServices;
public class W {
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref POINT p);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int c);
  [DllImport("user32.dll")] public static extern bool MoveWindow(IntPtr h, int x, int y, int w, int hh, bool r);
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr hdc, uint flags);
  public struct RECT { public int L, T, R, B; }
  public struct POINT { public int X, Y; }
}
"@
[W]::SetProcessDPIAware() | Out-Null
$p = Get-Process sketch-app -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowHandle -ne 0 } | Select-Object -First 1
if (-not $p) { Write-Output "NO WINDOW"; exit 1 }
$h = $p.MainWindowHandle
if (-not $NoResize) { [W]::ShowWindow($h, 9) | Out-Null }   # SW_RESTORE (keeps a maximized window as-is with -NoResize)
# Size the window so its CLIENT area is exactly Width x Height.
$wr = New-Object W+RECT; $cr = New-Object W+RECT
[W]::GetWindowRect($h, [ref]$wr) | Out-Null; [W]::GetClientRect($h, [ref]$cr) | Out-Null
$padW = ($wr.R - $wr.L) - ($cr.R - $cr.L); $padH = ($wr.B - $wr.T) - ($cr.B - $cr.T)
if (-not $NoResize) { [W]::MoveWindow($h, 20, 20, $Width + $padW, $Height + $padH, $true) | Out-Null }
Start-Sleep -Milliseconds 900
# PrintWindow(PW_RENDERFULLCONTENT) reads the window's OWN pixels even when it is
# covered by other windows -- no focus stealing, no capturing whatever is on top.
# (CopyFromScreen captured the user's browser when the app was not in front.)
[W]::GetWindowRect($h, [ref]$wr) | Out-Null; [W]::GetClientRect($h, [ref]$cr) | Out-Null
$ww = $wr.R - $wr.L; $wh = $wr.B - $wr.T
$whole = New-Object System.Drawing.Bitmap $ww, $wh
$gw = [System.Drawing.Graphics]::FromImage($whole)
$hdc = $gw.GetHdc()
$ok = [W]::PrintWindow($h, $hdc, 2)
$gw.ReleaseHdc($hdc); $gw.Dispose()
if (-not $ok) { Write-Output "PRINTWINDOW FAILED"; exit 2 }
$pt = New-Object W+POINT; $pt.X = 0; $pt.Y = 0; [W]::ClientToScreen($h, [ref]$pt) | Out-Null
$cw = $cr.R - $cr.L; $ch = $cr.B - $cr.T
$bmp = $whole.Clone((New-Object System.Drawing.Rectangle ($pt.X - $wr.L), ($pt.Y - $wr.T), $cw, $ch), $whole.PixelFormat)
$full = [System.IO.Path]::GetFullPath($Out)
New-Item -ItemType Directory -Force -Path ([System.IO.Path]::GetDirectoryName($full)) | Out-Null
$bmp.Save($full, [System.Drawing.Imaging.ImageFormat]::Png)
Write-Output "SAVED $full ${cw}x${ch}"
