# uia.ps1 - look at, or click a button in, the windows of the KeePassXC processes a test started (UI Automation).
#
# Purpose : testC opens a modal file dialog in KeePassXC (Invoke "Open Database") so that it cannot close at once;
#           "dump" lists the buttons, menu items and edit boxes of those windows to find the right names.
# Usage   : called by lib.mjs (uia); by hand:
#             powershell -NoProfile -ExecutionPolicy Bypass -File uia.ps1 -Action dump   -Pids 1234
#             powershell -NoProfile -ExecutionPolicy Bypass -File uia.ps1 -Action invoke -Pids 1234 -Name "Open Database"
# Needs   : Windows PowerShell 5.1. Pass ONLY ids of KeePassXC processes the test started, never the owner's own.
param([string]$Action, [string]$Pids, [string]$Name = "")
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
$AE = [System.Windows.Automation.AutomationElement]
$pidList = $Pids.Split(",") | Where-Object { $_ } | ForEach-Object { [int]$_ }
$tops = $AE::RootElement.FindAll([System.Windows.Automation.TreeScope]::Children, [System.Windows.Automation.Condition]::TrueCondition)
foreach ($t in $tops) {
  try { $p = $t.Current.ProcessId } catch { continue }
  if ($pidList -notcontains $p) { continue }
  if ($Action -eq "dump") {
    "TOP: [{0}] {1}" -f $t.Current.ControlType.ProgrammaticName, $t.Current.Name
    $all = $t.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)
    foreach ($e in $all) { $n = $e.Current.Name; $ct = $e.Current.ControlType.ProgrammaticName; if ($n -and ($ct -match "Button|MenuItem|Edit|CheckBox|Hyperlink|TabItem")) { "   {0} | {1}" -f $ct.Replace("ControlType.",""), $n } }
  } elseif ($Action -eq "invoke") {
    $c = New-Object System.Windows.Automation.PropertyCondition($AE::NameProperty, $Name)
    $e = $t.FindFirst([System.Windows.Automation.TreeScope]::Descendants, $c)
    if ($e) { try { $e.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke(); "invoked '$Name' in '$($t.Current.Name)'"; exit 0 } catch { "found '$Name' but Invoke failed: $_" } }
  }
}
if ($Action -eq "invoke") { "not found: $Name" }
