; Keep a desktop shortcut in both interactive and silent per-user installs.
; The framework helper also honors the installer's explicit no-shortcut option.
!macro NSIS_HOOK_POSTINSTALL
  Call CreateOrUpdateDesktopShortcut
!macroend
