Option Explicit
' Hidden launcher: relative paths; wait preserves the task instance and timeout.
Dim sh, fs, folder, config, code, dryArg
Set sh = CreateObject("WScript.Shell")
Set fs = CreateObject("Scripting.FileSystemObject")
folder = fs.GetParentFolderName(WScript.ScriptFullName)
config = fs.BuildPath(folder, "config.ps1")
If WScript.Arguments.Count > 0 Then config = WScript.Arguments(0)
If InStr(config, Chr(34)) > 0 Or InStr(config, vbCr) > 0 Or InStr(config, vbLf) > 0 Then WScript.Quit 2
dryArg = ""
If WScript.Arguments.Count > 1 Then
  If WScript.Arguments(1) = "--dry-run" Then dryArg = " -DryRun"
End If
sh.CurrentDirectory = folder
code = sh.Run("powershell.exe -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File " & Chr(34) & fs.BuildPath(folder, "stopline-guard.ps1") & Chr(34) & " -ConfigPath " & Chr(34) & config & Chr(34) & dryArg, 0, True)
WScript.Quit code
