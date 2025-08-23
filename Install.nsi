!define APP_NAME "ShinKrypt"
!define EXE_NAME "ShinKrypt.exe"
!define SET_NAME "Settings.ron"
!define LNK_NAME "ShinKrypt.lnk"
!define AUTHOR "Game Hacking Dojo"

# Set the name of the installer
Name "${APP_NAME}"

# Set the output file name (the finished installer.exe)
OutFile "Install ${EXE_NAME}"

# Set the default installation directory
# InstallDir "$PROGRAMFILES64\Author\App"
InstallDir "$APPDATA\${AUTHOR}\${APP_NAME}"

# Request application privileges for Windows Vista and newer
# Required for writing to HKCR
RequestExecutionLevel admin

# Pages to show during the installation
Page directory
Page instfiles

# Define a variable for the full path to the executable
# This makes it easier to reference and ensures the path is always quoted correctly
Var /GLOBAL AppPath

Var StartMenuFolder

Section "Install"

    # --- CRITICAL CHANGE 2: Use the 64-bit registry view ---
    # This ensures HKCR writes go to the native 64-bit registry, not the WOW6432Node.
    SetRegView 64

    # Set the output path to the installation directory.
    SetOutPath $INSTDIR

    # --- Add your files ---
    # Replace "App.exe" with your actual executable name
    File "${EXE_NAME}"
    # File "OtherFile.txt" # Your second file

    # Store the full, quoted path to the executable in our variable
    # This handles paths with spaces correctly
    StrCpy $AppPath "$INSTDIR\${EXE_NAME}"

    # Create Start Menu folder and shortcut
    StrCpy $StartMenuFolder "$SMPROGRAMS\${APP_NAME}"
    CreateDirectory "$StartMenuFolder"
    CreateShortcut "$StartMenuFolder\${LNK_NAME}" "$INSTDIR\${EXE_NAME}"

    # Create Desktop shortcut
    CreateShortcut "$DESKTOP\${LNK_NAME}" "$INSTDIR\${EXE_NAME}"

    # --- Create Context Menu for FILES (*) ---
    WriteRegStr HKCR "*\shell\${APP_NAME}" "" "${APP_NAME}"
    WriteRegStr HKCR "*\shell\${APP_NAME}" "Icon" $AppPath
    WriteRegStr HKCR "*\shell\${APP_NAME}\command" "" '$AppPath "%1"'

    # --- Create Context Menu for DIRECTORIES ---
    WriteRegStr HKCR "Directory\shell\${APP_NAME}" "" "${APP_NAME}"
    WriteRegStr HKCR "Directory\shell\${APP_NAME}" "Icon" $AppPath
    WriteRegStr HKCR "Directory\shell\${APP_NAME}\command" "" '$AppPath "%1"'

    # Create uninstaller
    WriteUninstaller "$INSTDIR\Uninstall.exe"

    # Add entry to Add/Remove Programs
    WriteRegStr HKLM "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "DisplayName" "${APP_NAME}"
    WriteRegStr HKLM "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "DisplayIcon" "$INSTDIR\${EXE_NAME},0"
    WriteRegStr HKLM "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "UninstallString" '"$INSTDIR\Uninstall.exe"'
    WriteRegStr HKLM "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "Publisher" "${AUTHOR}"

SectionEnd

Section "Uninstall"

    # --- Switch to 64-bit view for clean uninstallation ---
    SetRegView 64

    # Delete installed files
    Delete "$INSTDIR\${EXE_NAME}"
    Delete "$INSTDIR\${SET_NAME}"

    # Delete "$INSTDIR\OtherFile.txt"
    Delete "$INSTDIR\Uninstall.exe"

    # Remove the installation directory if it's empty
    RMDir "$INSTDIR"

    # Remove Start Menu shortcut and folder
    Delete "$SMPROGRAMS\${APP_NAME}\${LNK_NAME}"
    RMDir "$SMPROGRAMS\${APP_NAME}"

    # Remove Desktop shortcut
    Delete "$DESKTOP\${LNK_NAME}"

    # --- REMOVE Context Menu Entries ---
    # Delete the entire registry tree for the file context menu
    DeleteRegKey HKCR "*\shell\${APP_NAME}"
    # Delete the entire registry tree for the directory context menu
    DeleteRegKey HKCR "Directory\shell\${APP_NAME}"

    # Remove the uninstall information from Add/Remove Programs
    DeleteRegKey HKLM "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}"

SectionEnd
