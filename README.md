# mdread

Open a markdown file in a clean reading window. GitHub-flavored formatting, native GPU rendering (GPUI Kit), in-app font/size/theme controls, and live reload so a tiled plan stays in sync while an agent edits it.

## Setup

The project uses [mise](https://mise.jdx.dev) for the Rust toolchain.

```bash
git clone https://github.com/robsonperassoli/mdread.git
cd mdread
mise trust
mise install
```

Linux needs a working **Vulkan** loader (`libvulkan1`) and fontconfig. WebKitGTK is no longer required.

## Use

Dev loop (unoptimized):

```bash
mise run dev
mise run open -- ~/.cursor/plans/your-plan.plan.md
```

Install the current tree on this machine (`~/.local/bin/mdread`):

```bash
mise run install
mdread ~/.cursor/plans/some-plan.plan.md
```

On Omarchy, also register the theme/font hooks and the app menu entry:

```bash
mise run install && mise run setup-omarchy
```

`mise run install` puts the binary on `PATH`. `mise run setup-omarchy` copies `contrib/omarchy/` into place: the `mdread.toml.tpl` template, `theme-set` / `font-set` hooks, and a `.desktop` launcher with icon. After that, `omarchy theme set` and `omarchy font set` rewrite `~/.config/mdread/config.toml`; mdread reloads it. Opening **mdread** from the app menu (or running `mdread` with no file) shows a file picker. The installed binary returns to the shell as soon as the window is up; pass `--foreground` to keep it attached.

Leave that window tiled next to the terminal agent. On tiling window managers (Hyprland, Sway, i3, niri, …) mdread hides the title bar — close/minimize/maximize are dead weight because the compositor already owns the window. Floating desktops keep the title bar so you can drag and close. Override with `--decorations always|never` or `decorations` in the config file. When the file changes on disk (a plan update, a README rewrite), the reader re-renders. If you are near the bottom, it follows new content. YAML frontmatter is hidden, so Cursor plan files stay readable.

## Reading controls

The **Aa** button opens:

- Theme: bundled GPUI Kit themes, plus **custom**
- Font: **System** (platform UI font) or any installed family
- Size
- Radius and shadows
- When **custom** is selected: color pickers for background, foreground, muted, border, and primary

Choices are saved in `~/.config/mdread/config.toml`. The window reloads that file when it changes, so another program can recolor mdread without the reader knowing about it.

```toml
theme = "custom"
# font omitted → system UI font
size = 18.0
radius = 6.0
radius_lg = 8.0
shadow = true
decorations = "auto" # auto | always | never

[custom]
mode = "dark"
background = "#0d1117"
foreground = "#f0f6fc"
muted = "#8b949e"
border = "#30363d"
primary = "#4493f8"
```

`auto` hides the title bar on tiling window managers. Omarchy theme hooks write `theme = "custom"` and the `[custom]` palette. A saved `sepia` or GitHub `light`/`dark` config from the old WebView build still loads.
