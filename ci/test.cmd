if [%APPVEYOR_REPO_TAG%]==[true] (
    exit 0
)

cargo build --target %TARGET%
cargo test --target %TARGET%
cargo run --target %TARGET% --bin %CRATE_NAME%

target\%TARGET%\debug\install --install

C:\Users\appveyor\AppData\Local\fastjump\bin\fastjump.exe -h
C:\Users\appveyor\AppData\Local\fastjump\bin\j
C:\Users\appveyor\AppData\Local\fastjump\bin\j -s
C:\Users\appveyor\AppData\Local\fastjump\bin\j --add home
C:\Users\appveyor\AppData\Local\fastjump\bin\j -s
cd C:\Users\appveyor
C:\Users\appveyor\AppData\Local\fastjump\bin\j -s
cd C:\projects\fastjump

target\%TARGET%\debug\install --uninstall --purge
