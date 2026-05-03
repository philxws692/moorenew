## [1.1.1] - 2026-05-03

### 🐛 Bug Fixes

- Error with buzz_sync

### ⚙️ Miscellaneous Tasks

- Bump version to v1.1.1
## [1.1.0] - 2026-05-03

### 🚀 Features

- Add notification functionality
- Add configuration option for containers

### 🐛 Bug Fixes

- Correct systemd service file
- *(tests)* Make sha256 test mocked

### 🚜 Refactor

- Add better error handling

### ⚙️ Miscellaneous Tasks

- Make clippy happy
- Add justfile recipes for fmt and testing
- Release v1.1.0
## [1.0.2] - 2026-04-30

### 🐛 Bug Fixes

- Execute command the correct way

### ⚙️ Miscellaneous Tasks

- Release v1.0.2
## [1.0.1] - 2026-04-30

### 🐛 Bug Fixes

- Docker restart on local machine

### ⚙️ Miscellaneous Tasks

- Release v1.0.1
## [1.0.0] - 2026-01-08

### 🚀 Features

- Connecting and downloading via sftp works
- Added first cli tool functionality
- Ssh keygen functionality is working
- Ssh keygen configuration of filename and comment
- Added configuration generation
- Added info for copying ssh key
- Added file comparison
- Downloading certificates from reverse proxy
- Added sftp disconnect after file download
- Logging level configurable
- Added file trait extension
- Added remote sha256 command
- Added remote sha256 command
- Checking file hashed before downloading
- *(loki)* Added loki logging functionality
- Added auth configuration for loki
- Added postfix and dovecot restart
- Added service config file generation
- *(cli)* Updated cli to be more simple
- Better error messages
- *(config)* Moved config format to toml
- *(logging)* Added logging to file
- Add restart of nginx container

### 🐛 Bug Fixes

- Changed output of logging info to debug
- Extended configuration command
- Correct output location
- Added error handling
- Changed wrong dry run check
- *(logging)* Added small delay so that last logs get correctly sent
- *(logging)* Added host field for loki
- Added new line at the end of service configurations
- *(config)* Setup error handling
- Made dry run work correctly
- *(config)* Moved config loading further down
- Sshkeygen error on windows

### 💼 Other

- Added dependency

### 🚜 Refactor

- Moved logging to own module
- Better error handling for downloading
- Get env variables from env
- Removed unused imports
- Renamed sftp client to ssh client
- Moved download cert
- Restructured logging and cli behavior
- Moved sysinfo into new module
- Renamed services.rs to service.rs
- Made clippy happy
- Refactored project structure

### 🎨 Styling

- Ran cargo fmt

### ⚙️ Miscellaneous Tasks

- Changed .gitignore
- Some .gitignore adjustments
- Added justfile to ease compilation process
- Fixed unused imports
- Updated README.md
- Added build_deb workflow to build and publish to apt repository
- Changed to newer debian version for libssl3
- *(gitignore)* Added build directory
- *(README)* Added docker group requirement
- *(README)* Updated README.md
- *(README)* Added more planned features
- Added pre commit hooks
- Added github workflows
- Changed deb release workflow
- Updated README.md with beautiful logo
- Updated gitignore and pre commit config
- Updated README.md with planned features
- *(gitignore)* Added build directory
- Updated README.md with planned features
- Typo in README.md
- Updated versions
- Update pre-commit hooks
- Update crates
- Added cargo dist for releases
- Added .DS_Store file to .gitignore
- Release version 1.0.0
- Update cargo dist
