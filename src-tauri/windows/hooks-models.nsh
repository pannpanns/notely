!include "LogicLib.nsh"
!define NOTELY_CLEANUP_SOURCE "${__FILEDIR__}\cleanup-models.ps1"

!macro NOTELY_STOP_ENGINE OPTIONS
  Push $0
  Push $1
  Push $2
  InitPluginsDir
  File "/oname=$PLUGINSDIR\notely-cleanup.ps1" "${NOTELY_CLEANUP_SOURCE}"
  ; Prefer native PowerShell when running inside a 32-bit NSIS process.
  StrCpy $2 "$SYSDIR\WindowsPowerShell\v1.0\powershell.exe"
  IfFileExists "$WINDIR\Sysnative\WindowsPowerShell\v1.0\powershell.exe" 0 +2
    StrCpy $2 "$WINDIR\Sysnative\WindowsPowerShell\v1.0\powershell.exe"
  nsExec::ExecToStack '"$2" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$PLUGINSDIR\notely-cleanup.ps1" -InstallDir "$INSTDIR" ${OPTIONS}'
  Pop $0
  Pop $1
  ${If} $0 != 0
    MessageBox MB_OK|MB_ICONEXCLAMATION "Notely could not finish automatic cleanup. Your files were preserved. Please retry.$\r$\n$1" /SD IDOK
    Pop $2
    Pop $1
    Pop $0
    Abort
  ${EndIf}
  Pop $2
  Pop $1
  Pop $0
!macroend

!macro NSIS_HOOK_PREINSTALL
  !insertmacro NOTELY_STOP_ENGINE ""
  !insertmacro NOTELY_STOP_ENGINE "-RestoreModels"
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; Keep is the default, including silent upgrades/uninstalls.
  MessageBox MB_YESNO|MB_ICONQUESTION|MB_DEFBUTTON1 "Keep model data? / Simpan data model AI dan transkripsi agar dapat digunakan saat instal ulang? Pilih No untuk menghapus model. Rekaman tetap disimpan." /SD IDYES IDYES notely_keep_models
  !insertmacro NOTELY_STOP_ENGINE "-RemoveModels"
  Goto notely_models_done
  notely_keep_models:
  !insertmacro NOTELY_STOP_ENGINE "-KeepModels"
  notely_models_done:
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  !insertmacro NOTELY_STOP_ENGINE "-RestoreModels"
!macroend
