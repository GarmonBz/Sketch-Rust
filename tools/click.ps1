# tools/click.ps1 — post a left click to the sketch-app window at CLIENT coordinates
# (window messages only: does not move the user's mouse or steal focus).
# Usage: powershell -File tools\click.ps1 -X 1285 -Y 14 [-Double]
# Prints window state after the click: rect, minimized, maximized, alive.
param([int]$X, [int]$Y, [switch]$Double, [int]$WaitMs = 700)
Add-Type @"
using System; using System.Runtime.InteropServices;
public class C {
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr h);
  [DllImport("user32.dll")] public static extern bool IsZoomed(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  public struct RECT { public int L, T, R, B; }
}
"@
[C]::SetProcessDPIAware() | Out-Null
$p = Get-Process sketch-app -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowHandle -ne 0 } | Select-Object -First 1
if (-not $p) { Write-Output "NO WINDOW"; exit 1 }
$h = $p.MainWindowHandle
$lp = [IntPtr](($Y -shl 16) -bor ($X -band 0xFFFF))
[C]::PostMessage($h, 0x0200, [IntPtr]0, $lp) | Out-Null   # WM_MOUSEMOVE
Start-Sleep -Milliseconds 80
[C]::PostMessage($h, 0x0201, [IntPtr]1, $lp) | Out-Null   # WM_LBUTTONDOWN
Start-Sleep -Milliseconds 60
[C]::PostMessage($h, 0x0202, [IntPtr]0, $lp) | Out-Null   # WM_LBUTTONUP
if ($Double) {
  Start-Sleep -Milliseconds 60
  [C]::PostMessage($h, 0x0203, [IntPtr]1, $lp) | Out-Null # WM_LBUTTONDBLCLK
  [C]::PostMessage($h, 0x0202, [IntPtr]0, $lp) | Out-Null
}
Start-Sleep -Milliseconds $WaitMs
$alive = -not $p.HasExited -and (Get-Process -Id $p.Id -ErrorAction SilentlyContinue)
$r = New-Object C+RECT; [C]::GetWindowRect($h, [ref]$r) | Out-Null
Write-Output ("alive={0} minimized={1} maximized={2} rect={3},{4} {5}x{6}" -f [bool]$alive, [C]::IsIconic($h), [C]::IsZoomed($h), $r.L, $r.T, ($r.R - $r.L), ($r.B - $r.T))
