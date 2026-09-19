#!/bin/sh
if ! command -v arm-kindlehf-linux-gnueabihf-gcc &> /dev/null; then
    echo "Error: Please install koxtoolchain (https://github.com/koreader/koxtoolchain/releases)" >&2
    exit 1
fi

if ! command -v cargo-zigbuild &> /dev/null; then
    echo "Error: Please install cargo-zigbuild (see README)" >&2
    exit 1
fi

build_local() {
    echo "Jobman | Building locally..."
    cargo run
}

SYSROOT="$HOME/x-tools/arm-kindlehf-linux-gnueabihf/arm-kindlehf-linux-gnueabihf/sysroot"
export PATH="$HOME/x-tools/arm-kindlehf-linux-gnueabihf/bin:$PATH" # Just in case
build_kindle() {
    echo "Jobman | Building for Kindle..."
    echo "Cleaning up..."
    cd companion/dropbear && git reset --hard HEAD && git clean -fd && cd ../..
    cd companion/openssh && git reset --hard HEAD && git clean -fd && cd ../..
    rm -rf build
    mkdir -p build/jobman/bin

    # Step 1
    echo "Jobman | Building OpenSSH SFTP server"
    cd companion/openssh
    autoreconf
    
    ./configure --verbose \
        --host=arm-kindlehf-linux-gnueabihf \
        --without-openssl --without-zlib \
        CC="arm-kindlehf-linux-gnueabihf-gcc --sysroot=$SYSROOT" \
        LDFLAGS="-static"

    sed -i 's/-fzero-call-used-regs=used//g' Makefile
    sed -i 's/-fzero-call-used-regs=used//g' openbsd-compat/Makefile

    make -j$(nproc) sftp-server CC="arm-kindlehf-linux-gnueabihf-gcc --sysroot=$SYSROOT"
    arm-kindlehf-linux-gnueabihf-strip sftp-server
    cp sftp-server ../../build/jobman/bin/sftp-server
    rm -f ../.config_sftp_stamp
    cd ../..

    # Step 2
    echo "Jobman | Patching and building Dropbear"
    cd companion/dropbear/src
    patch -p1 -l < ../../dropbear.patch

    cd ..

    ./configure --verbose \
        --host=arm-kindlehf-linux-gnueabihf \
        --disable-syslog --disable-pam --disable-shadow --disable-zlib \
        CC="arm-kindlehf-linux-gnueabihf-gcc --sysroot=$SYSROOT" \
        CFLAGS="-DFAKE_ROOT -Wno-deprecated" \
        LDFLAGS="-static"

    make -j$(nproc) PROGRAMS="dropbear dbclient scp" MULTI=1 \
        CC="arm-kindlehf-linux-gnueabihf-gcc --sysroot=$SYSROOT"
    
    arm-kindlehf-linux-gnueabihf-strip dropbearmulti
    cp dropbearmulti ../../build/jobman/bin/dropbearmulti
    cd ../..

    # Step 3
    echo "Jobman | Building Jobman"
    RUST_FONTCONFIG_DLOPEN=1 cargo zigbuild --release --target armv7-unknown-linux-musleabihf
    cp target/armv7-unknown-linux-musleabihf/release/jobman build/jobman
    arm-kindlehf-linux-gnueabihf-strip build/jobman

    # Step 4
    echo "Jobman | Copying additional distribution files"
    cp -r companion/dist/* build/

    echo "Jobman | Kindle build finished"
}

case "$1" in
    "local")   build_local ;;
    "kindle")  build_kindle ;;
    "")        build_local ;;
    *)         echo "Jobman | Unknown argument: $1" ;;
esac

