 #!/bin/bash
# check THEOS settings
if [ -z "$THEOS" ]; then
    echo "THEOS environment variable not set"
    exit 1
fi

if [ ! -d "$THEOS/lib" ]; then
    echo "THEOS lib [$THEOS/lib] not exists"
    exit 1
fi

PROJ_ROOT=$(pwd)
BUILD_LIB=0
BUILD_PACKAGE=0
INSTALL_PACKAGE=0
RELEASE_BUILD=0
TOOL_NAME=""

for arg in "$@"
do
    case $arg in 
        l | lib)
            BUILD_LIB=1
            shift
        ;;
        p | package)
            BUILD_PACKAGE=1
            shift
        ;;
        i | install)
            INSTALL_PACKAGE=1
            shift
        ;;
        r | release)
            RELEASE_BUILD=1
            shift
        ;;
        c | clean)
            ./.scripts/clean.sh $PROJ_ROOT
            exit 0
        ;;
        *)
            TOOL_NAME=$arg
            shift
        ;;
    esac
done

if [ $BUILD_LIB -eq 1 ]; then
    if [ -z $TOOL_NAME ]; then
        echo "No tool name given"
        exit 1
    elif [ $RELEASE_BUILD -eq 1 ]; then
        ./.scripts/gen_staticlib.sh "release" $TOOL_NAME
    else
        ./.scripts/gen_staticlib.sh "debug" $TOOL_NAME
    fi
fi

if [ $BUILD_PACKAGE -eq 1 ]; then
    cd $PROJ_ROOT/theos_code
    if [ $RELEASE_BUILD -eq 1 ]; then
        make package FINALPACKAGE=1
    else
        make package
    fi
    cd $PROJ_ROOT
fi

if [ $INSTALL_PACKAGE -eq 1 ]; then
    cd $PROJ_ROOT/theos_code
    make install
    cd $PROJ_ROOT
fi