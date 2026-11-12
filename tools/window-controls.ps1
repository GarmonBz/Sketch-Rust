# target/ctl.ps1 — exercise the title-bar window controls of sketch-app.exe
# (expects SKETCH_DPR=1 so client px == CSS px).
Add-Type @"
using System; using System.Runtime.InteropServices;
public class C {
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool MoveWindow(IntPtr h, int x, int y, int w, int hh, bool r);
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr h);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int c);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr a, int x, int y, int cx, int cy, uint f);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern bool GetCursorPos(out POINT p);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, int dx, int dy, uint d, IntPtr e);
  public struct RECT { public int L, T, R, B; }
  public struct POINT { public int X, Y; }
}
"@
[C]::SetProcessDPIAware() | Out-Null
function Win { (Get-Process sketch-app -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowHandle -ne 0 } | Select-Object -First 1) }
function Rect($h) { $r = New-Object C+RECT; [C]::GetWindowRect($h, [ref]$r) | Out-Null; "$($r.L),$($r.T) $($r.R - $r.L)x$($r.B - $r.T)" }
function Click($h, $x, $y) {
  # Real input: topmost + foreground, move, press, release; cursor restored.
  $r = New-Object C+RECT; [C]::GetWindowRect($h, [ref]$r) | Out-Null
  $old = New-Object C+POINT; [C]::GetCursorPos([ref]$old) | Out-Null
  [C]::SetWindowPos($h, [IntPtr](-1), 0, 0, 0, 0, 0x0003) | Out-Null
  [C]::SetForegroundWindow($h) | Out-Null; Start-Sleep -Milliseconds 250
  [C]::SetCursorPos($r.L + $x, $r.T + $y) | Out-Null; Start-Sleep -Milliseconds 200
  [C]::mouse_event(0x0002, 0, 0, 0, [IntPtr]0); Start-Sleep -Milliseconds 80
  [C]::mouse_event(0x0004, 0, 0, 0, [IntPtr]0); Start-Sleep -Milliseconds 200
  [C]::SetWindowPos($h, [IntPtr](-2), 0, 0, 0, 0, 0x0003) | Out-Null
  [C]::SetCursorPos($old.X, $old.Y) | Out-Null
  Start-Sleep -Milliseconds 900
}
$p = Win; if (-not $p) { "NO WINDOW"; exit 1 }
$h = $p.MainWindowHandle
[C]::MoveWindow($h, 20, 20, 1400, 860, $true) | Out-Null; Start-Sleep -Milliseconds 500
"start            rect=$(Rect $h) iconic=$([C]::IsIconic($h))"

Click $h 1285 15
"after Minimize   iconic=$([C]::IsIconic($h))"
[C]::ShowWindow($h, 9) | Out-Null; Start-Sleep -Milliseconds 700
"restored         rect=$(Rect $h) iconic=$([C]::IsIconic($h))"

Click $h 1331 15
"after FullScreen rect=$(Rect $h)"
$fr = New-Object C+RECT; [C]::GetWindowRect($h, [ref]$fr) | Out-Null
Click $h ($fr.R - $fr.L - 69) 15
"after FullScreen#2 rect=$(Rect $h)"
[C]::MoveWindow($h, 20, 20, 1400, 860, $true) | Out-Null; Start-Sleep -Milliseconds 500

# Drag-to-move needs real input (the OS move loop tracks the physical
# button): topmost + SendInput-style mouse_event, cursor restored after.
$old = New-Object C+POINT; [C]::GetCursorPos([ref]$old) | Out-Null
[C]::SetWindowPos($h, [IntPtr](-1), 0, 0, 0, 0, 0x0003) | Out-Null   # HWND_TOPMOST, NOMOVE|NOSIZE
[C]::SetForegroundWindow($h) | Out-Null; Start-Sleep -Milliseconds 300
[C]::SetCursorPos(620, 35) | Out-Null; Start-Sleep -Milliseconds 150
[C]::mouse_event(0x0002, 0, 0, 0, [IntPtr]0); Start-Sleep -Milliseconds 200   # LEFTDOWN
foreach ($i in 1..10) { [C]::SetCursorPos(620 + 12 * $i, 35 + 6 * $i) | Out-Null; Start-Sleep -Milliseconds 40 }
Start-Sleep -Milliseconds 150
[C]::mouse_event(0x0004, 0, 0, 0, [IntPtr]0); Start-Sleep -Milliseconds 300   # LEFTUP
[C]::SetWindowPos($h, [IntPtr](-2), 0, 0, 0, 0, 0x0003) | Out-Null   # HWND_NOTOPMOST
[C]::SetCursorPos($old.X, $old.Y) | Out-Null
"after drag       rect=$(Rect $h)   (expected origin ~140,80)"

Click $h 1376 15
Start-Sleep -Milliseconds 800
$alive = Get-Process -Id $p.Id -ErrorAction SilentlyContinue
"after Close      process alive=$([bool]$alive)"
