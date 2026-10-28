; Extra steps for the Notaya setup (referenced by bundle.windows.nsis.installerHooks).
;
; An installed Notaya keeps its notes in %LOCALAPPDATA%\Notaya Notes\data, not in the program folder (%LOCALAPPDATA%\Notaya),
; and the uninstaller never removes that folder. This tells the person so, once, at the end of
; an interactive uninstall.

; Not shown for a silent uninstall, a passive one, or the uninstall step of an upgrade
; (a newer setup run over an old install uninstalls the old program first, with /UPDATE).
!macro NSIS_HOOK_POSTUNINSTALL
  IfSilent notaya_no_note
  ${If} $UpdateMode = 1
  ${OrIf} $PassiveMode = 1
    Goto notaya_no_note
  ${EndIf}
  MessageBox MB_OK|MB_ICONINFORMATION "Notaya was removed from this PC.$\r$\n$\r$\nYour notes and their backups were NOT deleted. They are still in:$\r$\n$LOCALAPPDATA\Notaya Notes\data$\r$\n$\r$\nInstall Notaya again to keep using them, or delete that folder yourself if you no longer want them."
  notaya_no_note:
!macroend
