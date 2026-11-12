# tools/click-real.ps1 — REAL left click on the sketch-app window at CLIENT
# coordinates (iced ignores PostMessage'd clicks). Pins the window topmost,
# moves the cursor, presses + releases, then restores the cursor position and
# the window's z-order. Keep bursts short: this moves the user's mouse.
# Usage: powershell -NoProfile -ExecutionPolicy Bypass -File tools\click-real.ps1 -X 140 -Y 46
#        ... -Escape      (instead of clicking: send Escape to the window, after
#                          a click at -X/-Y if given, which gives it focus)
#        ... -HoverOnly   (move there and stay ~WaitMs, no click; cursor restored)
param([int]$X = -1, [int]$Y = -1, [switch]$Escape, [switch]$HoverOnly, [int]$WaitMs = 600)
Add-Type @"
using System; using System.Runtime.InteropServices;
public class R {
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref POINT p);
  [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr a, int x, int y, int cx, int cy, uint f);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern bool GetCursorPos(out POINT p);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, int dx, int dy, uint d, IntPtr e);
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint f, IntPtr e);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  public struct POINT { public int X, Y; }
}
"@
[R]::SetProcessDPIAware() | Out-Null
$p = Get-Process sketch-app -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowHandle -ne 0 } | Select-Object -First 1
if (-not $p) { Write-Output "NO WINDOW"; exit 1 }
$h = $p.MainWindowHandle
$old = New-Object R+POINT; [R]::GetCursorPos([ref]$old) | Out-Null
[R]::SetWindowPos($h, [IntPtr](-1), 0, 0, 0, 0, 0x0013) | Out-Null   # TOPMOST, NOMOVE|NOSIZE|NOACTIVATE
[R]::SetForegroundWindow($h) | Out-Null
Start-Sleep -Milliseconds 150
try {
  if ($X -ge 0 -and $Y -ge 0) {
    $pt = New-Object R+POINT; $pt.X = $X; $pt.Y = $Y
    [R]::ClientToScreen($h, [ref]$pt) | Out-Null
    # Approach from 1px away so the app sees a CursorMoved onto the target.
    [R]::SetCursorPos($pt.X - 1, $pt.Y) | Out-Null; Start-Sleep -Milliseconds 60
    [R]::SetCursorPos($pt.X, $pt.Y) | Out-Null; Start-Sleep -Milliseconds 120
    if (-not $HoverOnly) {
      [R]::mouse_event(0x0002, 0, 0, 0, [IntPtr]0); Start-Sleep -Milliseconds 70   # LEFTDOWN
      [R]::mouse_event(0x0004, 0, 0, 0, [IntPtr]0)                                 # LEFTUP
    }
    Start-Sleep -Milliseconds $WaitMs
  }
  if ($Escape) {
    # Never send a key to the user's window: require a real click inside the
    # app first (-X/-Y), then verify the app actually holds the foreground.
    if ($X -lt 0 -or $Y -lt 0) { Write-Output "REFUSED: -Escape needs -X/-Y (a click inside the app to focus it)"; exit 3 }
    if ([R]::GetForegroundWindow() -ne $h) { Write-Output "REFUSED: app is not the foreground window; Escape not sent"; exit 4 }
    [R]::keybd_event(0x1B, 0, 0, [IntPtr]0); Start-Sleep -Milliseconds 50
    [R]::keybd_event(0x1B, 0, 2, [IntPtr]0); Start-Sleep -Milliseconds 300
  }
} finally {
  [R]::SetCursorPos($old.X, $old.Y) | Out-Null
  [R]::SetWindowPos($h, [IntPtr](-2), 0, 0, 0, 0, 0x0013) | Out-Null  # NOTOPMOST
}
Write-Output "clicked client $X,$Y escape=$Escape hoverOnly=$HoverOnly"
