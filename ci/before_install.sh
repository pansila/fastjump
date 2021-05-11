

common_install() {
  curl -O https://github.com/nushell/nushell/releases/download/0.30.0/nu_0_30_0_linux.tar.gz
  tar -xf nu_0_30_0_linux.tar.gz
}

install_for_linux() {
    echo 111
}

install_for_macos() {
    echo 111
}

if [ $TRAVIS_OS_NAME = linux ]; then
    install_for_linux
else
    install_for_macos
fi
