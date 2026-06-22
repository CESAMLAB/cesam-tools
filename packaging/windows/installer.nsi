; Installeur Windows générique des instruments cesam-tools (ORME, OSNE…).
; Construit depuis Linux par `makensis` (paquet `nsis`), piloté par
; scripts/make-installers.sh via des défines -D :
;
;   makensis -DBIN=osne -DPRODNAME=OSNE -DVERSION=0.1.0 \
;            -DSRCEXE=dist/osne-windows-x86_64.exe \
;            -DICO=dist/_installer/osne.ico \
;            -DOUTFILE=dist/osne-setup-x86_64.exe \
;            packaging/windows/installer.nsi
;
; Produit un setup .exe : installe l'exécutable, crée les raccourcis (menu
; Démarrer + bureau) et un désinstalleur, enregistre l'entrée « Programmes et
; fonctionnalités ».

!ifndef BIN
  !error "BIN non défini (-DBIN=...)"
!endif
!ifndef PRODNAME
  !define PRODNAME "${BIN}"
!endif
!ifndef VERSION
  !define VERSION "0.0.0"
!endif
!ifndef OUTFILE
  !define OUTFILE "${BIN}-setup-x86_64.exe"
!endif

Unicode true
Name "${PRODNAME} ${VERSION}"
OutFile "${OUTFILE}"
InstallDir "$PROGRAMFILES64\${PRODNAME}"
InstallDirRegKey HKLM "Software\${PRODNAME}" "InstallDir"
RequestExecutionLevel admin
ShowInstDetails show
ShowUninstDetails show

!ifdef ICO
  Icon "${ICO}"
  UninstallIcon "${ICO}"
!endif

!define UNINSTKEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODNAME}"

Page directory
Page instfiles
UninstPage uninstConfirm
UninstPage instfiles

Section "Install"
  SetOutPath "$INSTDIR"
  File "/oname=${BIN}.exe" "${SRCEXE}"
!ifdef ICO
  File "/oname=${BIN}.ico" "${ICO}"
  !define SHORTCUT_ICON "$INSTDIR\${BIN}.ico"
!else
  !define SHORTCUT_ICON "$INSTDIR\${BIN}.exe"
!endif

  ; Raccourcis menu Démarrer + bureau (icône de marque).
  CreateDirectory "$SMPROGRAMS\${PRODNAME}"
  CreateShortcut "$SMPROGRAMS\${PRODNAME}\${PRODNAME}.lnk" "$INSTDIR\${BIN}.exe" "" "${SHORTCUT_ICON}"
  CreateShortcut "$DESKTOP\${PRODNAME}.lnk" "$INSTDIR\${BIN}.exe" "" "${SHORTCUT_ICON}"

  ; Désinstalleur + entrée « Programmes et fonctionnalités ».
  WriteUninstaller "$INSTDIR\uninstall.exe"
  WriteRegStr HKLM "Software\${PRODNAME}" "InstallDir" "$INSTDIR"
  WriteRegStr HKLM "${UNINSTKEY}" "DisplayName" "${PRODNAME}"
  WriteRegStr HKLM "${UNINSTKEY}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKLM "${UNINSTKEY}" "Publisher" "CESAM-Lab"
  WriteRegStr HKLM "${UNINSTKEY}" "UninstallString" "$\"$INSTDIR\uninstall.exe$\""
  WriteRegStr HKLM "${UNINSTKEY}" "DisplayIcon" "${SHORTCUT_ICON}"
  WriteRegDWORD HKLM "${UNINSTKEY}" "NoModify" 1
  WriteRegDWORD HKLM "${UNINSTKEY}" "NoRepair" 1
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\${BIN}.exe"
  Delete "$INSTDIR\${BIN}.ico"
  Delete "$INSTDIR\uninstall.exe"
  Delete "$SMPROGRAMS\${PRODNAME}\${PRODNAME}.lnk"
  RMDir "$SMPROGRAMS\${PRODNAME}"
  Delete "$DESKTOP\${PRODNAME}.lnk"
  RMDir "$INSTDIR"
  DeleteRegKey HKLM "${UNINSTKEY}"
  DeleteRegKey HKLM "Software\${PRODNAME}"
SectionEnd
