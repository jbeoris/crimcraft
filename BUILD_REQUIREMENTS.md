# Build Requirements for CrimCraft

## Required Dependencies

You'll need to install ALSA development libraries to build this project:

- On Debian/Ubuntu: `sudo apt-get install libasound2-dev`
- On Fedora: `sudo dnf install alsa-lib-devel`
- On Arch: `sudo pacman -S alsa-lib`

## Windows WSL2 Setup

When building in WSL2 on Windows:

1. Install the required system libraries in your WSL2 distribution:
   ```bash
   sudo apt-get install libasound2-dev pkg-config
   ```

2. If you encounter `failed to run custom build command for alsa-sys`, ensure you've installed the ALSA development libraries as shown above.

## Troubleshooting

If you see the error: `pkg-config exited with status code 1` related to missing `alsa.pc`, it means you need to install the ALSA development libraries.
