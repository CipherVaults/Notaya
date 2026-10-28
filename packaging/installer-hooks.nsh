; Extra steps for the Notaya setup (referenced by bundle.windows.nsis.installerHooks).
;
; An installed Notaya keeps its notes in %LOCALAPPDATA%\Notaya Notes\data, not in the program folder (%LOCALAPPDATA%\Notaya),
; and the uninstaller never removes that folder. This tells the person so, once, at the end of
; an interactive uninstall.

; Not shown for a silent uninstall, a passive one, or the uninstall step of an upgrade.
; Two kinds of upgrade: a setup started with /UPDATE (the uninstaller gets /UPDATE too), and a setup
; window where the person keeps "Uninstall before installing" on the "Already Installed" page (the setup
; hides itself and runs the old uninstaller without /UPDATE). In the second case the setup window, titled
; "Notaya Setup", still exists, so its presence means "this is an upgrade, say nothing". The hook that
; runs is the one inside the installed version's own uninstaller, so this takes effect for upgrades
; from V1.3.0 on.
!macro NSIS_HOOK_POSTUNINSTALL
  IfSilent notaya_no_note
  ${If} $UpdateMode = 1
  ${OrIf} $PassiveMode = 1
    Goto notaya_no_note
  ${EndIf}
  FindWindow $0 "" "Notaya Setup"
  ${If} $0 <> 0
    Goto notaya_no_note
  ${EndIf}
  MessageBox MB_OK|MB_ICONINFORMATION "Notaya was removed from this PC.$\r$\n$\r$\nYour notes and their backups were NOT deleted. They are still in:$\r$\n$LOCALAPPDATA\Notaya Notes\data$\r$\n$\r$\nInstall Notaya again to keep using them, or delete that folder yourself if you no longer want them."
  notaya_no_note:
!macroend
