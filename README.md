```
 __   __     _     ____   ____  
 \ \ / /    / \   |  _ \ / ___| 
  \ V /    / _ \  | |_) |\___ \ 
   | |    / ___ \ |  _ <  ___) |
   |_|   /_/   \_\|_| \_\|____/ 

 ____  _______     _____ _   _  ____  _____ 
|  _ \| ____\ \   / / __| \ | |/ ___|| ___|
| |_) |  _|  \ \ / /| |_ | \| | |  _|  _| 
|  _ <| |___  \ V / | |__| |\ | |_| | |___ 
|_| \_\_____|  \_/  \____|_| \|\____|_____|
```

Recreating a version of the classic Yars revenge game.

Built using Rust.

Vibe coded using Claude since I currently do not know Rust at all!

https://en.wikipedia.org/wiki/Yars%27_Revenge

## How to Run

### Prerequisites

You need Rust and Cargo installed. Visit [rustup.rs](https://rustup.rs) and follow the instructions for your platform.

You will also need a C/C++ linker:

- **Windows** — Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) and select the "Desktop development with C++" workload
- **Mac** — Run `xcode-select --install` in a terminal
- **Linux (Ubuntu/Debian)** — Run `sudo apt install build-essential libgl1-mesa-dev`

### Clone and Run

```bash
git clone https://github.com/johnarmitage/YarsRevenge-Claude.git
cd YarsRevenge-Claude
cargo run
```

### Controls

| Key | Action |
|---|---|
| Arrow keys | Move Yar |
| Space | Fire missile (always) / Fire Zorlon Cannon (when charged) |

### How to Play

- Fly Yar toward the **yellow shield** on the right and touch it to nibble away blocks
- Nibbling a shield block **charges the Zorlon Cannon** — an icon appears on the left edge of the screen
- Press **Space** when the cannon is charged to fire it across the screen
- The cannon destroys whatever it hits first: a shield block, the Swirl, or the Qotile
- You must nibble a clear path through the shield before the cannon can reach the Qotile
- The **neutral zone** (blue stripe in the center) is a safe area — the Swirl cannot kill you there
- Yar wraps around the top and bottom of the screen
- Avoid the **Swirl** — the homing projectile launched by the Qotile
