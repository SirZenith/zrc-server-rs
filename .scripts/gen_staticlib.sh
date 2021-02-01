#!/bin/bash
######################################################
# build static lib for iOS using golib
######################################################
set -e

BUILD_VERSION=$1
TOOL_NAME=$2
PROJ_ROOT=$(pwd)
MAKE_PATH=$PROJ_ROOT/theos_code
LIB_PATH=$PROJ_ROOT/target/aarch64-apple-ios
LIB_NAME=lib$TOOL_NAME.a

mkdir -p $LIB_PATH

echo "building darwin/arm64 static lib ($BUILD_VERSION)"
if [ "$BUILD_VERSION" == "release" ]; then
    cargo build --lib --target=aarch64-apple-ios --release
else
    cargo build --lib --target=aarch64-apple-ios
fi

if [ ! -f $LIB_PATH/$LIB_NAME ] && [ ! -f $LIB_PATH/$BUILD_VERSION/$LIB_NAME ]; then
    echo "failed to build darwin/arm64 static lib!"
    exit 1
fi

######################################################
# build debian binary for iOS using theos
######################################################
# Makefile of .deb package
cd $MAKE_PATH
echo 'include $(THEOS)/makefiles/common.mk

export ARCHS = arm64

TOOL_NAME = '$TOOL_NAME'
'$TOOL_NAME'_FILES = main.mm
'$TOOL_NAME'_LDFLAGS = '$LIB_PATH/$LIB_NAME'

include $(THEOS_MAKE_PATH)/tool.mk
' > ./Makefile

rm -rf .theos
