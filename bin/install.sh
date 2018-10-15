#!/usr/bin/env bash

set -e

bin="`dirname "$0"`"
ROOT_DIR="`cd "$bin/../"; pwd`"

# Name of the OS
OS_TYPE="unknown"
# Whether or not OS is supported
OS_SUPPORTED=""

case "$OSTYPE" in
  solaris*) OS_TYPE="SOLARIS";;
  darwin*) OS_TYPE="OSX";;
  linux*) OS_TYPE="LINUX";;
  bsd*) OS_TYPE="BSD";;
  msys*) OS_TYPE="WINDOWS";;
  *) OS_TYPE="unknown: $OSTYPE";;
esac

if [ "$OS_TYPE" = "OSX" ]; then
  OS_SUPPORTED="1"
elif [ "$OS_TYPE" = "LINUX" ]; then
  OS_SUPPORTED="1"
else
  OS_SUPPORTED="0"
fi

if [ -z "$OS_SUPPORTED" ]; then
  echo "Unsupported OS: $OS_TYPE"
  exit 1
fi

if [[ -z $(which cargo) ]]; then
  echo "Cargo binary is not found, please install Rust and try again"
  exit 1
fi

cd $ROOT_DIR/server && cargo build --release
