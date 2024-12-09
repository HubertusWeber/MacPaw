#!/bin/bash

# =============================================================================
# macOS Privacy and Performance Configuration Script
#
# Purpose: Configures various macOS settings to enhance privacy, improve
# performance, and create a minimalist experience by disabling unnecessary
# features and services.
# =============================================================================

# -----------------------------------------------------------------------------
# Audio Settings
# Disables various system sounds and audio feedback for a quieter experience
# -----------------------------------------------------------------------------

# Disable startup sound
sudo nvram SystemAudioVolume=%80

# Disable UI sound effects
defaults write com.apple.systemsound "com.apple.sound.uiaudio.enabled" -int 0
defaults write -g com.apple.sound.uiaudio.enabled -bool false

# Disable volume change feedback
defaults write -g com.apple.sound.beep.feedback -bool false

# Disable Finder sounds (like emptying trash)
defaults write com.apple.finder FinderSounds -bool false

# Mute system audio
osascript -e "set volume output muted true"

# Disable sound when charging cable is plugged in
defaults write com.apple.PowerChime ChimeOnNoHardware -bool true
killall PowerChime

# -----------------------------------------------------------------------------
# Privacy and Security Settings
# Enhances privacy by disabling various tracking and data collection features
# -----------------------------------------------------------------------------

# Disable location services system-wide
sudo defaults write /var/db/locationd/Library/Preferences/ByHost/com.apple.locationd LocationServicesEnabled -int 0

# Disable remote Apple events
sudo systemsetup -setremoteappleevents off

# Disable various diagnostic and analytics features
sudo defaults write /Library/Application\ Support/CrashReporter/DiagnosticMessagesHistory.plist AutoSubmit -bool false
defaults write com.apple.CrashReporter DialogType none
sudo defaults write /Library/Application\ Support/com.apple.security.profilemanager SubmitDiagInfo -bool false
defaults write com.apple.appstore SendProductTelemetry -bool false

# -----------------------------------------------------------------------------
# Sharing and Connectivity
# Disables various sharing and connectivity features for enhanced privacy
# -----------------------------------------------------------------------------

# Disable AirDrop
defaults write com.apple.NetworkBrowser DisableAirDrop -bool true

# Disable printer sharing
sudo cupsctl --no-share-printers

# Disable Handoff functionality
defaults write ~/Library/Preferences/ByHost/com.apple.coreservices.useractivityd.plist ActivityAdvertisingAllowed -bool false
defaults write ~/Library/Preferences/ByHost/com.apple.coreservices.useractivityd.plist ActivityReceivingAllowed -bool false

# -----------------------------------------------------------------------------
# Siri and Search Settings
# Disables Siri and limits search functionality for privacy
# -----------------------------------------------------------------------------

# Disable Siri completely
defaults write com.apple.assistant.support "Assistant Enabled" -bool false
defaults write com.apple.Siri StatusMenuVisible -bool false
defaults write com.apple.Siri UserHasDeclinedEnable -bool true
defaults write com.apple.systemuiserver "NSStatusItem Visible Siri" -bool false

# Disable Spotlight suggestions
defaults write com.apple.safari UniversalSearchEnabled -bool false

# -----------------------------------------------------------------------------
# User Interface Settings
# Configures UI elements for minimalism and privacy
# -----------------------------------------------------------------------------

# Disable screen saver
defaults -currentHost write com.apple.screensaver idleTime 0

# Disable Quick Look previews
defaults write com.apple.finder QLInlinePreviewDisabled -bool true

# Disable all hot corners
defaults write com.apple.dock wvous-tl-corner -int 0
defaults write com.apple.dock wvous-tr-corner -int 0
defaults write com.apple.dock wvous-bl-corner -int 0
defaults write com.apple.dock wvous-br-corner -int 0

# Disable iCloud Drive warnings
defaults write com.apple.finder FXEnableRemoveFromICloudDriveWarning -bool false

# Disable Safari search suggestions
defaults write com.apple.Safari SuppressSearchSuggestions -bool true

# -----------------------------------------------------------------------------
# Restart Services
# Restart necessary services to apply changes
# -----------------------------------------------------------------------------

killall Finder
killall Dock
killall SystemUIServer
