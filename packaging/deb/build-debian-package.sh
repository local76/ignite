#!/bin/sh
# Resolve script directory and change to it
cd "$(dirname "$0")"

echo "Building Debian package..."

# Create staging directory structure
mkdir -p debian/usr/bin
mkdir -p debian/usr/share/applications
mkdir -p debian/usr/share/pixmaps
mkdir -p ../../dist/packages

# Locate and copy binary
if [ -f "../../dist/binaries/rstart" ]; then
    cp ../../dist/binaries/rstart debian/usr/bin/rstart
elif [ -f "../../target/x86_64-unknown-linux-musl/release/rstart" ]; then
    cp ../../target/x86_64-unknown-linux-musl/release/rstart debian/usr/bin/rstart
elif [ -f "../../target/release/rstart" ]; then
    cp ../../target/release/rstart debian/usr/bin/rstart
else
    echo "Error: compiled rstart binary not found in target/ or dist/binaries/."
    exit 1
fi

chmod 755 debian/usr/bin/rstart

# Copy desktop file and icon
cp ../desktop/rstart.desktop debian/usr/share/applications/rstart.desktop
cp ../../assets/brand/app_icon.png debian/usr/share/pixmaps/rstart.png
chmod 644 debian/usr/share/applications/rstart.desktop
chmod 644 debian/usr/share/pixmaps/rstart.png

# Run dpkg-deb to build the package
dpkg-deb --build debian ../../dist/packages/rstart.deb

# Clean up staging files
rm -f debian/usr/bin/rstart
rm -rf debian/usr/share
