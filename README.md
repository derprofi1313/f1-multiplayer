# F1 Multiplayer 2016–2026

Two friends, one career: F1 25 with every season from 2016 to 2026, raced together through a server that runs on the host's PC. No EA servers are involved.

The launcher is based on Swofty Launcher (AGPL-3.0). Addons made for Swofty work here too.

## What it does

- **Your own server.** The host starts it on the Multiplayer page and gets a six-digit server code. On the same network, friends type only the code. Over the internet they use the host code, which contains the address and a certificate fingerprint.
- **Rooms and LAN races** through the host's server, with an encrypted relay and an addon fingerprint check.
- **Multiplayer career** in F1 25's main menu, without the online lock.
- **Addons**, installed from the Addon Browser. The catalog is read from this repository's `catalog/` folder, so new mods appear without a launcher update.
- **Test friend** (`f1mp mp test-friend <code> --probe`) joins your server as a simulated second player, so the server can be tested on one PC.

## Build

On Windows, with the Rust toolchain `1.95.0-x86_64-pc-windows-gnullvm` and llvm-mingw on `PATH`:

```
cargo build -p swofty-cli -p swofty-launcher
```

This builds `f1mp.exe` and `f1-multiplayer.exe`. A release build needs the Windows SDK (for `fxc.exe`).

The game-side DLL is built with `native/build.sh`, which needs Zig.

Data lives in `%LOCALAPPDATA%\F1Multiplayer`. Set `F1MP_DATA_DIR` to use another folder.

## Publishing a mod

Mods are published to the catalog without a launcher update:

```
python catalog/publish.py <addon folder> --repo <owner>/<name>
```

The script packs the addon, adds its version to `catalog/v1/addons`, and prints the GitHub CLI commands that upload the packed file as a release asset. Then commit and push `catalog/v1/addons`.

## Licence

F1 Multiplayer is distributed under the GNU Affero General Public License, version 3 or later. See `LICENSE`.

F1 Multiplayer is not affiliated with Swofty, EA, Codemasters or Formula One. "F1" and "Formula 1" are trademarks of Formula One Licensing B.V.
