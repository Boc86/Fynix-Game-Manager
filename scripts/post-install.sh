#!/bin/bash
# Post-install script for Fynix GM
# Checks for required system dependencies and UMU launcher

set -e

# Check for UMU launcher (optional but recommended for Wine games)
if ! command -v umu-launcher &> /dev/null; then
    echo "Note: umu-launcher is not installed."
    echo "Install it to enable Wine/Proton game launching:"
    echo "  Arch: yay -S umu-launcher"
    echo "  Ubuntu/Debian: wget -O /usr/local/bin/umu-launcher https://github.com/Open-Wine-Components/umu-launcher/releases/latest/download/umu-run && chmod +x /usr/local/bin/umu-launcher"
    echo "  Nix: nix-shell -p umu-launcher"
fi

# Check for Steam (optional)
if ! command -v steam &> /dev/null; then
    echo "Note: Steam is not installed."
    echo "Install Steam for Steam game detection:"
    echo "  Arch: sudo pacman -S steam"
    echo "  Ubuntu: sudo apt install steam"
fi

# Check for Heroic (optional)
if ! command -v heroic &> /dev/null; then
    echo "Note: Heroic Games Launcher is not installed."
    echo "Install Heroic for Epic/GOG game detection:"
    echo "  Arch: yay -S heroic-games-launcher"
    echo "  Ubuntu: wget -O heroic.deb https://github.com/heroiclabs/heroeslauncher/releases/latest/download/heroic-games-launcher.deb && sudo apt install ./heroic.deb"
fi

# Check for Lutris (optional)
if ! command -v lutris &> /dev/null; then
    echo "Note: Lutris is not installed."
    echo "Install Lutris for Linux game detection:"
    echo "  Arch: sudo pacman -S lutris"
    echo "  Ubuntu: sudo apt install lutris"
fi

echo "Fynix GM installation complete."
