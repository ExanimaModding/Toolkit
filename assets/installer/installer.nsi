!define APP_NAME "Exanima Modding Toolkit"
!define COMP_NAME "Exanima Modding"
!define VERSION "${CARGO_PKG_VERSION}"
!define PRODUCT_VERISON "${PRODUCT_VERSION}"
!define WEB_SITE "https://codeberg.org/ExanimaModding/Toolkit"
!define DESCRIPTION "A toolkit for modding Exanima"
!define INSTALLER_NAME "EMTK Installer - ${VERSION}.exe"
!define MAIN_APP_EXE "emtk.exe"
!define ICON "..\images\corro.ico"

!define INSTALL_DIR "$LOCALAPPDATA\${APP_NAME}"
!define INSTALL_TYPE "SetShellVarContext all"
!define REG_ROOT "HKLM"
!define REG_APP_PATH "Software\Microsoft\Windows\CurrentVersion\App Paths\${MAIN_APP_EXE}"
!define UNINSTALL_PATH "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}"
!define REG_START_MENU "Start Menu Folder"

!define SM_FOLDER "${APP_NAME}"

## Set Executable Metadata
VIProductVersion  "${PRODUCT_VERISON}"
VIAddVersionKey "ProductName" "${APP_NAME}"
VIAddVersionKey "CompanyName" "${COMP_NAME}"
VIAddVersionKey "LegalCopyright" "${COPYRIGHT}"
VIAddVersionKey "FileDescription" "${DESCRIPTION}"
VIAddVersionKey "FileVersion" "${VERSION}"

!define MUI_ICON "${ICON}"
!define MUI_UNICON "${ICON}"
; !define MUI_WELCOMEFINISHPAGE_BITMAP "${BANNER}"
; !define MUI_UNWELCOMEFINISHPAGE_BITMAP "${BANNER}"

## Set Installer Metadata
SetCompressor /SOLID Lzma
Name "${APP_NAME}"
Caption "${APP_NAME}"
OutFile "..\..\target\release\${INSTALLER_NAME}"
BrandingText "${APP_NAME}"
InstallDirRegKey "${REG_ROOT}" "${REG_APP_PATH}" ""
InstallDir "${INSTALL_DIR}"


######################################################################

!include "MUI2.nsh"

!define MUI_ABORTWARNING
!define MUI_UNABORTWARNING

!insertmacro MUI_PAGE_WELCOME

!ifdef LICENSE_TXT
	!insertmacro MUI_PAGE_LICENSE "${LICENSE_TXT}"
!endif

!insertmacro MUI_PAGE_DIRECTORY

!insertmacro MUI_PAGE_INSTFILES

!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM

!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_UNPAGE_FINISH

!insertmacro MUI_LANGUAGE "English"

######################################################################

Section -MainProgram
	${INSTALL_TYPE}

	SetOverwrite ifnewer
	SetOutPath "$INSTDIR"
	File "..\..\target\release\emtk.exe"
	File "..\..\target\release\emf.dll"
	File "..\..\target\release\emf.dll.lib"
SectionEnd

######################################################################

Section -Icons_Reg
	SetOutPath "$INSTDIR"
	WriteUninstaller "$INSTDIR\uninstall.exe"

	!ifdef REG_START_MENU
		CreateDirectory "$SMPROGRAMS\${SM_FOLDER}"
		CreateShortCut "$SMPROGRAMS\${SM_FOLDER}\${APP_NAME}.lnk" "$INSTDIR\${MAIN_APP_EXE}"
		CreateShortCut "$DESKTOP\${APP_NAME}.lnk" "$INSTDIR\${MAIN_APP_EXE}"
		CreateShortCut "$SMPROGRAMS\${SM_FOLDER}\Uninstall ${APP_NAME}.lnk" "$INSTDIR\uninstall.exe"
	!endif

	!ifndef REG_START_MENU
		CreateDirectory "$SMPROGRAMS\${APP_NAME}"
		CreateShortCut "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk" "$INSTDIR\${MAIN_APP_EXE}"
		CreateShortCut "$DESKTOP\${APP_NAME}.lnk" "$INSTDIR\${MAIN_APP_EXE}"
		CreateShortCut "$SMPROGRAMS\${APP_NAME}\Uninstall ${APP_NAME}.lnk" "$INSTDIR\uninstall.exe"
	!endif

	WriteRegStr ${REG_ROOT} "${REG_APP_PATH}" "" "$INSTDIR\${MAIN_APP_EXE}"
	WriteRegStr ${REG_ROOT} "${UNINSTALL_PATH}"  "DisplayName" "${APP_NAME}"
	WriteRegStr ${REG_ROOT} "${UNINSTALL_PATH}"  "UninstallString" "$INSTDIR\uninstall.exe"
	WriteRegStr ${REG_ROOT} "${UNINSTALL_PATH}"  "DisplayIcon" "$INSTDIR\${MAIN_APP_EXE}"
	WriteRegStr ${REG_ROOT} "${UNINSTALL_PATH}"  "DisplayVersion" "${VERSION}"
	WriteRegStr ${REG_ROOT} "${UNINSTALL_PATH}"  "Publisher" "${COMP_NAME}"
SectionEnd

######################################################################

Section Uninstall
	${INSTALL_TYPE}

	RmDir /r "$INSTDIR"

	!ifdef REG_START_MENU
		Delete "$SMPROGRAMS\${SM_FOLDER}\${APP_NAME}.lnk"
		Delete "$SMPROGRAMS\${SM_FOLDER}\Uninstall ${APP_NAME}.lnk"
		Delete "$DESKTOP\${APP_NAME}.lnk"

		RmDir "$SMPROGRAMS\${SM_FOLDER}"
	!endif

	!ifndef REG_START_MENU
			Delete "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk"
			Delete "$SMPROGRAMS\${APP_NAME}\Uninstall ${APP_NAME}.lnk"
		Delete "$DESKTOP\${APP_NAME}.lnk"

		RmDir "$SMPROGRAMS\${APP_NAME}"
	!endif

	DeleteRegKey ${REG_ROOT} "${REG_APP_PATH}"
	DeleteRegKey ${REG_ROOT} "${UNINSTALL_PATH}"
SectionEnd

