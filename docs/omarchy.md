# Omarchy

mdread only reads `~/.config/mdread/config.toml`. The hooks in `contrib/omarchy/` are optional: they write that file when you change the Omarchy theme or font, and they add an app menu entry.

Install the binary first, then run setup.

From a GitHub release archive (`contrib/omarchy` is included):

```bash
tar -xzf mdread-x86_64-linux.tar.gz
install -Dm755 mdread-x86_64-linux/mdread ~/.local/bin/mdread
mdread-x86_64-linux/contrib/omarchy/setup
```

From a git checkout:

```bash
mise run install
mise run setup-omarchy
```

`setup` requires `omarchy` on `PATH`. It installs:

- `mdread.toml.tpl` into Omarchy's themed config directory, so `omarchy theme set` fills mdread's colors
- `theme-set` and `font-set` hooks
- a `.desktop` launcher and the `mdread` icon, so the app menu can open markdown files

The theme hook sets `theme = "custom"` and writes the `[custom]` palette from the current Omarchy colors. The font hook stores the desktop font. mdread reloads `config.toml` on its own.

Opening **mdread** from the app menu, or running `mdread` with no file, shows a file picker.
