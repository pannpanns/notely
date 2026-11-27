; Notely v2.0.0 — 100% Pure Native Rust Desktop Installer (NSIS)
; Compact single-file setup packaging notely-desktop.exe + notely-engine.exe + Python runtime.
; Reuses hooks-models.nsh so running processes are stopped cleanly and NotelyModels is preserved.

Unicode true
!include "MUI2.nsh"
!include "FileFunc.nsh"
!include "..\src-tauri\windows\hooks-models.nsh"

!ifndef APP_VERSION
  !define APP_VERSION "2.0.0"
!endif

!define APP_NAME "Notely"
!define APP_PUBLISHER "Notely Contributors"
!define APP_EXE "notely-desktop.exe"
!define UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\Notely"

Name "${APP_NAME} ${APP_VERSION}"
OutFile "..\dist-native\Notely_${APP_VERSION}_x64-setup.exe"
InstallDir "$LOCALAPPDATA\Notely"
InstallDirRegKey HKCU "Software\Notely" "InstallDir"
RequestExecutionLevel user
SetCompressor /SOLID lzma

!define MUI_ICON "..\src-tauri\icons\notely-icon.ico"
!define MUI_UNICON "..\src-tauri\icons\notely-icon.ico"
!define MUI_ABORTWARNING
!define MUI_FINISHPAGE_RUN "$INSTDIR\${APP_EXE}"
!define MUI_FINISHPAGE_RUN_TEXT "Jalankan Notely sekarang"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "Indonesian"
!insertmacro MUI_LANGUAGE "English"

Section "Notely (Wajib)" SecMain
  SectionIn RO

  ; Stop any running Notely / engine instance cleanly and restore preserved models if present
  !insertmacro NSIS_HOOK_PREINSTALL

  SetOutPath "$INSTDIR"

  ; Native Rust Desktop UI executable
  File "..\src-native\target\release\notely-desktop.exe"

  ; Compact PyInstaller AI sidecar + Python runtime DLLs
  File /r "..\dist\notely-engine\*.*"

  ; Write uninstaller
  WriteUninstaller "$INSTDIR\uninstall.exe"

  ; Shortcuts
  CreateDirectory "$SMPROGRAMS\${APP_NAME}"
  CreateShortcut "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk" "$INSTDIR\${APP_EXE}" "" "$INSTDIR\${APP_EXE}" 0
  CreateShortcut "$SMPROGRAMS\${APP_NAME}\Uninstall ${APP_NAME}.lnk" "$INSTDIR\uninstall.exe"
  CreateShortcut "$DESKTOP\${APP_NAME}.lnk" "$INSTDIR\${APP_EXE}" "" "$INSTDIR\${APP_EXE}" 0

  ; Registry metadata for Windows Apps & Features
  WriteRegStr HKCU "Software\Notely" "InstallDir" "$INSTDIR"
  WriteRegStr HKCU "${UNINST_KEY}" "DisplayName" "${APP_NAME}"
  WriteRegStr HKCU "${UNINST_KEY}" "DisplayVersion" "${APP_VERSION}"
  WriteRegStr HKCU "${UNINST_KEY}" "Publisher" "${APP_PUBLISHER}"
  WriteRegStr HKCU "${UNINST_KEY}" "DisplayIcon" "$INSTDIR\${APP_EXE}"
  WriteRegStr HKCU "${UNINST_KEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "${UNINST_KEY}" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegStr HKCU "${UNINST_KEY}" "QuietUninstallString" '"$INSTDIR\uninstall.exe" /S'
  WriteRegDWORD HKCU "${UNINST_KEY}" "NoModify" 1
  WriteRegDWORD HKCU "${UNINST_KEY}" "NoRepair" 1

  ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
  IntFmt $0 "0x%08X" $0
  WriteRegDWORD HKCU "${UNINST_KEY}" "EstimatedSize" "$0"
SectionEnd

Section "Uninstall"
  ; Safely stop active recordings/processes and protect NotelyModels
  !insertmacro NSIS_HOOK_PREUNINSTALL

  Delete "$DESKTOP\${APP_NAME}.lnk"
  RMDir /r "$SMPROGRAMS\${APP_NAME}"

  DeleteRegKey HKCU "${UNINST_KEY}"
  DeleteRegKey HKCU "Software\Notely"

  ; Remove installed binaries and runtime files
  RMDir /r "$INSTDIR"

  ; Restore kept NotelyModels folder if user chose to keep models (default in silent mode)
  !insertmacro NSIS_HOOK_POSTUNINSTALL
SectionEnd
