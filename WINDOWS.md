## Prerequisites
- Windows
  - **Install WSL (Windows Subsystem for Linux)**: WebKit is primarily built for Linux/macOS, so we will use WSL to create a Linux environment on Windows.
    - To install WSL, run the following command in PowerShell (as Administrator):
      ```bash
      wsl.exe --install
      ```
    - After installation, restart your machine, and install your preferred Linux distribution (e.g., Ubuntu) from the Microsoft Store.
    - Once installed, open the WSL terminal and update the system:
      ```bash
      sudo apt update && sudo apt upgrade
      ```
  - **Install Dependencies on WSL**:
    - Install the necessary build dependencies within your WSL terminal:
      ```bash
      sudo apt install -y build-essential cmake git python3 python3-pip \
      libwebkit2gtk-4.1-dev gir1.2-webkit2-4.0
      ```
  - **Clone WebKit Repository**:
    - Clone the WebKit source code:
      ```bash
      git clone https://github.com/WebKit/webkit.git
      ```
    - Navigate to the WebKit directory:
      ```bash
      cd webkit
      ```
  - **Install JSC (JavaScriptCore)**:
    - JSC is part of WebKit and will be compiled during the build process. No separate installation is needed.
  - **Build WebKit**:
    - Configure the build using CMake:
      ```bash
      cmake -G "Unix Makefiles" .
      ```
    - Compile WebKit:
      ```bash
      make
      ```
    - Install WebKit:
      ```bash
      sudo make install
      ```
  - **Using WebKit**:
    - Once WebKit is installed in the WSL environment, you can begin using it for development. For Rust development, you can link to WebKit’s GTK bindings within your WSL setup.

**Note:** This method leverages WSL to run WebKit in a Linux-like environment on Windows. While WebKit does not natively support Windows, this workaround allows for full WebKit development capabilities.
