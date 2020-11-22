#!/bin/bash
PROJ_ROOT=$1
cargo clean
rm -rf $PROJ_ROOT/theos_code/.theos
rm -rf $PROJ_ROOT/theos_code/packages
