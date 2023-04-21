# `bioristor-lib`

This repository contains the following packages:
* `bioristor-lib`: library that implements the algorithms for solving the mathematical model that describes the behavior of the Bioristor sensor for embedded devices (`no_std` packages);
* `nucleo-f767zi`: example of application of the `bioristor-lib` library to a [NUCLEO-F767ZI](https://www.st.com/en/evaluation-tools/nucleo-f767zi.html) board;
* `nucleo-l476rg`: example of application of the `bioristor-lib` library to a [NUCLEO-L476RG](https://www.st.com/en/evaluation-tools/nucleo-l476rg.html) board;
* `profiler`: library that implements a profiler based on `SysTick` for Cortex-M microcontrollers.


## Table of Contents

* [Development](#development)
  * [Developing inside a Container](#developing-inside-a-container)
  * [Manual Setup](#manual-setup)
  * [Build](#build)
  * [Tests](#tests)
* [Authors](#authors)


## Development

For contributing to this project, you can either choose to work in a [development container](#developing-inside-a-container) or [manually setup the enviroment](#manual-setup).

The use of a development container is strongly recommended since it allows the developers to work in the same environment and to make no additional effort to setup all the tools required during the development.

### Developing inside a Container

Prerequisites:
  - Install [Docker](https://docs.docker.com/get-docker/), [Visual Studio Code](https://code.visualstudio.com/) and [Dev Containers](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers) extension
  - Read the documentation at [Developing inside a Container](https://code.visualstudio.com/docs/remote/containers)

In a nutshell, you need to:
  1. Clone the repository with `git clone https://github.com/dsg-unipr/bioristor-lib.git`.
  1. Start VS Code and run **Dev Containers: Open Folder in Container...** from the Command Palette.

Or:
  1. Start VS Code and run **Dev Containers: Clone Repository in Container Volume...** from the Command Palette.
  1. Enter `https://github.com/dsg-unipr/bioristor-lib.git` in the input box that appears and press `Enter`.

The VS Code window will reload, clone the source code, and start building the dev container. A progress notification provides status updates.
After the build completes, VS Code will automatically connect to the container.

### Manual Setup

To edit this project, you must first make a clone of the repository:

```
git clone https://github.com/dsg-unipr/bioristor-lib.git
```

Install various dependencies needed to connect and debug development boards:

```bash
apt-get update
apt-get -y install --no-install-recommends \
    libudev-dev \
    libcapstone4 \
    libftdi1-dev \
    libgpiod2 \
    libhidapi-hidraw0 \
    libjaylink0 \
    libjim0.79 \
    libusb-0.1-4 \
    libusb-1.0-0-dev \
    pkg-config
# Install multi-arch version of objdump and nm, and create symbolic links.
apt-get -y install --no-install-recommends binutils-multiarch
ln -s /usr/bin/objdump /usr/bin/objdump-multiarch
ln -s /usr/bin/nm /usr/bin/nm-multiarch
# Install GDB for ARM systems.
apt-get -y install --no-install-recommends gdb-arm-none-eabi
# Install OpenOCD for debbuging.
apt-get -y install --no-install-recommends openocd
```

Then, follow the instructions in [Install Rust](https://www.rust-lang.org/tools/install) to download [Rustup](https://github.com/rust-lang/rustup), the Rust toolchain installer, and setup `cargo` and `rustc`.

Make sure you are using the latest stable version of Rust:
```
rustup toolchain install stable
rustup default stable
```

Install some usefull `cargo` components:
```
rustup component add llvm-tools-preview
cargo install cargo-binutils
cargo install cargo-flash
cargo install probe-run
rustup component add clippy rustfmt
```

Finally, install the Rust Toolchain for Cortex-M4 and Cortex-M7 with hardware floating point:

```
rustup target add thumbv7em-none-eabihf
```

### Build

To build the project, run the following command:
```
cargo build [--release]
```
This will generate an executable program in the folder `target/debug` or `target/release`, depending on which build profile was selected. Use the `release` profile if you care about performance.

### Tests

To execute all the unit/integration tests implemented in the project, run the following command:
```
cargo test
```


## Authors

- Francesco Saccani (francesco.saccani@unipr.it)
