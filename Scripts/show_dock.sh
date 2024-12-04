#!/bin/bash
# show_dock.sh
# This script restores the dock to normal functionality

# Reset auto-hide settings
defaults delete com.apple.dock autohide
defaults delete com.apple.dock autohide-delay
defaults delete com.apple.dock autohide-time-modifier

# Reset dock size
defaults delete com.apple.dock tilesize

# Reset all other modifications
defaults delete com.apple.dock static-only
defaults delete com.apple.dock showhidden
defaults delete com.apple.dock size-immutable
defaults delete com.apple.dock hide-mirror
defaults delete com.apple.dock no-bouncing
defaults delete com.apple.dock mineffect

# Set dock position to bottom
defaults write com.apple.dock orientation -string "bottom"

# Restart the Dock for changes to take effect
killall Dock
