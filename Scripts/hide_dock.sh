#!/bin/bash
# hide_dock.sh
# This script aggressively disables the dock

# Enable auto-hide
defaults write com.apple.dock autohide -bool true

# Set an extremely long auto-hide delay (effectively prevents showing)
defaults write com.apple.dock autohide-delay -float 1000

# Set a huge auto-hide time modifier (makes animation extremely slow)
defaults write com.apple.dock autohide-time-modifier -float 1000

# Minimize dock size to smallest possible
defaults write com.apple.dock tilesize -int 1

# Disable the dock completely
defaults write com.apple.dock static-only -bool true
defaults write com.apple.dock showhidden -bool true
defaults write com.apple.dock size-immutable -bool true
defaults write com.apple.dock hide-mirror -bool true
defaults write com.apple.dock no-bouncing -bool true
defaults write com.apple.dock mineffect -string "scale"

# Restart the Dock for changes to take effect
killall Dock
