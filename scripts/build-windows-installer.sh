#!/bin/bash
WXS_FILE="wix/main.wxs"
HALLOY_VERSION=$(cat VERSION)

# build the binary
scripts/build-windows.sh

# install latest wix
dotnet tool install --global wix

# add required wix extensions
wix extension add WixToolset.UI.wixext
wix extension add WixToolset.Firewall.wixext

# build the installer
wix build -pdbtype none -arch x64 -d PackageVersion=$HALLOY_VERSION $WXS_FILE -o target/release/halloy-installer.msi -ext WixToolset.UI.wixext -ext WixToolset.Firewall.wixext
