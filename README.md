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

Publishable release (binary + `.deb` under `src-tauri/target/release/`):

```bash
mise run build
```

Leave that window tiled next to the terminal agent. On tiling window managers (Hyprland, Sway, i3, niri, …) mdread hides the GTK title bar — close/minimize/maximize are dead weight because the compositor already owns the window. Floating desktops keep the title bar so you can drag and close. Override with `--decorations always|never` or the **Aa** panel. When the file changes on disk (a plan update, a README rewrite), the reader re-renders. If you are near the bottom, it follows new content. YAML frontmatter is hidden, so Cursor plan files stay readable.

## Reading controls

The **Aa** button opens:

- Font: serif, sans, mono, or a custom family name
- Size
- Title bar: auto (hide on tiling WMs), always show, or always hide
- Colors: GitHub light, GitHub dark, sepia, or custom background/text

Choices are saved in `~/.config/mdread/config.toml`.
