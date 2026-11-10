' Starts the Notaya heartbeat tick with NO visible window (window style 0) and waits for it to end,
' so the scheduled task keeps its "one instance at a time" and 3-hour limit behaviour.
' Undo: run register-task.ps1 (the original, which launches powershell.exe directly).
Set sh = CreateObject("WScript.Shell")
sh.CurrentDirectory = "D:\Notaya-lanes\heartbeat"
code = sh.Run("powershell.exe -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File ""D:\Notaya-lanes\heartbeat\heartbeat.ps1""", 0, True)
WScript.Quit code