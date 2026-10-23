# kpwin.ps1 - list or politely close the windows of the KeePassXC processes a test started.
#
# Purpose : the Quit tests (testC, testE) must clean up KeePassXC WITHOUT force-stopping it.
#             -Action list   print the top-level windows of the given processes
#             -Action close  post WM_CLOSE to the visible ones (the same as clicking the X)
#             -Action keys   bring the visible window to the front and send keys, for example "{ESC}" or "{ENTER}",
#                            to answer a dialog such as "save changes?"
# Usage   : called by lib.mjs (kpw); by hand:
#             powershell -NoProfile -ExecutionPolicy Bypass -File kpwin.ps1 -Action list -Pids 1234,5678
# Needs   : Windows PowerShell 5.1. Pass ONLY the ids of KeePassXC processes the test started (lib.mjs
#           compares with a baseline taken before the test, so the owner's own KeePassXC is never listed).
# It never calls Stop-Process or Kill.
param([string]$Action, [string]$Pids)
Add-Type @"
using System; using System.Text; using System.Collections.Generic; using System.Runtime.InteropServices;
public class W {
  public delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc p, IntPtr l);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetClassName(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  public static List<IntPtr> ForPids(HashSet<uint> pids) {
    var r = new List<IntPtr>();
    EnumWindows((h, l) => { uint pid; GetWindowThreadProcessId(h, out pid); if (pids.Contains(pid)) r.Add(h); return true; }, IntPtr.Zero);
    return r;
  }
}
"@
$ids = New-Object 'System.Collections.Generic.HashSet[uint32]'
$Pids.Split(",") | ForEach-Object { if ($_) { [void]$ids.Add([uint32]$_) } }
if ($ids.Count -eq 0) { "no KeePassXC process"; exit 0 }
$wins = [W]::ForPids($ids)
foreach ($h in $wins) {
  $t = New-Object System.Text.StringBuilder 256; $c = New-Object System.Text.StringBuilder 256
  [void][W]::GetWindowText($h, $t, 256); [void][W]::GetClassName($h, $c, 256)
  $vis = [W]::IsWindowVisible($h)
  if ($Action -eq "list") { "{0}|visible={1}|class={2}|title={3}" -f $h, $vis, $c, $t }
  elseif ($Action -eq "close" -and $vis) { [void][W]::PostMessage($h, 0x0010, [IntPtr]::Zero, [IntPtr]::Zero); "WM_CLOSE -> $h ($t)" }
  elseif ($Action -eq "keys" -and $vis) { [void][W]::SetForegroundWindow($h); Start-Sleep -Milliseconds 300; Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.SendKeys]::SendWait($args[0]); "keys sent to $h ($t)" }
}
