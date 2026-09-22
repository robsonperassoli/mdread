# Development

mdread is a Rust binary. The toolchain is pinned in [`mise.toml`](../mise.toml) (Rust 1.98, including rustfmt and clippy).

```bash
git clone https://github.com/robsonperassoli/mdread.git
cd mdread
mise trust
mise install
```

Linux builds link `libxcb`, `libxkbcommon`, and `libxkbcommon-x11`. On Debian and Ubuntu:

```bash
sudo apt-get install -y build-essential pkg-config \
  libxkbcommon-dev libxkbcommon-x11-dev libxcb1-dev libx11-dev libwayland-dev
```

Running the app also needs a Vulkan loader and `fc-list` from fontconfig.

## Tasks

Open the sample file:

```bash
mise run dev
```

Open a specific file:

```bash
mise run open -- ~/.cursor/plans/your-plan.plan.md
```

Release-build and install to `~/.local/bin/mdread`:

```bash
mise run install
```

Omarchy theme and font hooks are separate. See [Omarchy](omarchy.md).

## Tests

```bash
cargo test
```

The tests are unit tests. They do not open a window.
