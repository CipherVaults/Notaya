' Starts the Notaya stop-line guard run with NO visible window (window style 0) and waits for it to end,
' so the scheduled task keeps its "one instance at a time" behaviour. Same pattern as heartbeat-hidden.vbs.
' Undo: Unregister-ScheduledTask -TaskName 'Notaya stop-line guard' -Confirm:$false
Set sh = CreateObject("WScript.Shell")
sh.CurrentDirectory = "D:\Notaya-lanes\heartbeat"
code = sh.Run("powershell.exe -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File ""D:\Notaya-lanes\heartbeat\stopline-guard.ps1""", 0, True)
WScript.Quit code
