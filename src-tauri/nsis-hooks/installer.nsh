; EidosInsight NSIS Installer Hooks
; Adds VC++ Redistributable installation after main app files are copied
; Note: Resources (vc_redist.x64.exe) are automatically included via tauri.conf.json > bundle.resources

!macro NSIS_HOOK_PREINSTALL
  ; Pre-install: nothing needed before file copy
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; Post-install: Check and install VC++ Redistributable if not present
  ; Resources have been copied to $INSTDIR\resources\ by Tauri's NSIS script
  ReadRegStr $0 HKLM "SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64" "Installed"
  ${If} $0 != "1"
    DetailPrint "Installing Visual C++ Redistributable..."
    ExecWait '"$INSTDIR\resources\vc_redist.x64.exe" /install /quiet /norestart' $1
    ${If} $1 = 0
      DetailPrint "Visual C++ Redistributable installed successfully."
    ${ElseIf} $1 = 3010
      DetailPrint "Visual C++ Redistributable installed successfully (reboot required)."
    ${Else}
      DetailPrint "Visual C++ Redistributable installation returned code $1. Continuing..."
    ${EndIf}
  ${Else}
    DetailPrint "Visual C++ Redistributable already installed."
  ${EndIf}
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; Pre-uninstall: nothing needed
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ; Post-uninstall: We don't uninstall VC++ runtime as other apps may need it
!macroend
