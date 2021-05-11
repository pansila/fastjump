## prepare test environment
# Get-ExecutionPolicy
# Set-ExecutionPolicy Bypass -Scope Process -Force; iex ((New-Object System.Net.WebClient).DownloadString('https://chocolatey.org/install.ps1'))
choco install clink -a | tail 50
choco install cmder -a | tail 50

# we don't run the "test phase" when doing deploys
If (-not (Test-Path env:APPVEYOR_REPO_TAG) {
  cargo run --target %TARGET% --bin install -- --install &&

  C:\Users\appveyor\AppData\Local\fastjump\bin\fastjump.exe -h
  C:\Users\appveyor\AppData\Local\fastjump\bin\j
  C:\Users\appveyor\AppData\Local\fastjump\bin\j -s
  C:\Users\appveyor\AppData\Local\fastjump\bin\j --add home
  C:\Users\appveyor\AppData\Local\fastjump\bin\j -s
  cd C:\Users\appveyor
  C:\Users\appveyor\AppData\Local\fastjump\bin\j -s
  cd C:\projects\fastjump

  cargo run --target %TARGET% --bin install -- --uninstall --purge
}
