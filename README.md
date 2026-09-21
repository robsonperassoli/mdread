# mdread

Open a markdown file in a clean reading window. GitHub-flavored formatting, in-app font/size/color controls, and live reload so a tiled plan stays in sync while an agent edits it.

## Setup

The project uses [mise](https://mise.jdx.dev) for the Rust toolchain and Tauri CLI.

```bash
git clone https://github.com/robsonperassoli/mdread.git
cd mdread
mise trust
mise install
```

Linux also needs WebKitGTK (already present on Omarchy as `webkit2gtk-4.1`).

## Use

Dev loop (unoptimized, live Tauri reload):

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

Publishable release (binary + `.deb` under `src-tauri/target/release/`):

```bash
mise run build
```

Leave that window tiled next to the terminal agent. On tiling window managers (Hyprland, Sway, i3, niri, …) mdread hides the GTK title bar — close/minimize/maximize are dead weight because the compositor already owns the window. Floating desktops keep the title bar so you can drag and close. Override with `--decorations always|never` or `decorations` in the config file. When the file changes on disk (a plan update, a README rewrite), the reader re-renders. If you are near the bottom, it follows new content. YAML frontmatter is hidden, so Cursor plan files stay readable.

## Reading controls

The **Aa** button opens:

- Font: any installed family
- Size
- Theme: **Light** or **Dark** (GitHub colors)

Choices are saved in `~/.config/mdread/config.toml`. The window reloads that file when it changes, so another program can recolor mdread without the reader knowing about it. The title bar is config-only:

```toml
decorations = "auto" # auto | always | never
```

`auto` hides it on tiling window managers. A saved `sepia` theme still renders, but it is no longer in the panel.
