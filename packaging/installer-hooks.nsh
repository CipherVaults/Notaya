; Extra steps for the Notaya setup (referenced by bundle.windows.nsis.installerHooks).
;
; An installed Notaya keeps its notes in %LOCALAPPDATA%\Notaya Notes\data, not in the program folder (%LOCALAPPDATA%\Notaya),
; and the uninstaller never removes that folder. This tells the person so, once, at the end of
; an interactive uninstall (a silent uninstall shows nothing).

!macro NSIS_HOOK_POSTUNINSTALL
  IfSilent +2
  MessageBox MB_OK|MB_ICONINFORMATION "Notaya was removed from this PC.$\r$\n$\r$\nYour notes and their backups were NOT deleted. They are still in:$\r$\n$LOCALAPPDATA\Notaya Notes\data$\r$\n$\r$\nInstall Notaya again to keep using them, or delete that folder yourself if you no longer want them."
!macroend
