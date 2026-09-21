const PRESETS = {
  light: { bg: "#ffffff", fg: "#1f2328" },
  dark: { bg: "#0d1117", fg: "#f0f6fc" },
  sepia: { bg: "#f4ecd8", fg: "#5b4636" },
};

const FONT_PRESETS = [
  'Georgia, "Iowan Old Style", Palatino, serif',
  'system-ui, "Segoe UI", sans-serif',
  'ui-monospace, "JetBrains Mono", "Iosevka", monospace',
];

const state = {
  font: FONT_PRESETS[0],
  size: 18,
  theme: "dark",
  bg: PRESETS.dark.bg,
  fg: PRESETS.dark.fg,
  decorations: "auto",
};

const $ = (id) => document.getElementById(id);

function api() {
  return window.__TAURI__;
}

function applyAppearance() {
  const root = document.documentElement;
  root.dataset.theme = state.theme === "custom" ? "custom" : state.theme;
  root.style.setProperty("--md-font", state.font);
  root.style.setProperty("--md-size", `${state.size}px`);
  const colors = state.theme === "custom" ? state : PRESETS[state.theme] || PRESETS.dark;
  root.style.setProperty("--md-bg", colors.bg);
  root.style.setProperty("--md-fg", colors.fg);

  const useLightCss = state.theme === "light" || state.theme === "sepia";
  $("gh-light").disabled = !useLightCss;
  $("gh-dark").disabled = useLightCss;

  $("size").value = String(state.size);
  $("bg").value = toHex(state.bg);
  $("fg").value = toHex(state.fg);

  const preset = FONT_PRESETS.includes(state.font);
  $("font").value = preset ? state.font : "custom";
  $("custom-font-wrap").hidden = preset;
  $("custom-font").value = state.font;
  $("custom-colors").hidden = state.theme !== "custom";
  $("decorations").value = state.decorations || "auto";

  for (const button of document.querySelectorAll("[data-theme]")) {
    button.setAttribute("aria-pressed", String(button.dataset.theme === state.theme));
  }
}

function toHex(color) {
  if (/^#[0-9a-fA-F]{6}$/.test(color)) return color;
  return "#888888";
}

function renderDocument(doc, follow) {
  const article = $("content");
  const nearBottom =
    window.innerHeight + window.scrollY >= document.body.scrollHeight - 96;
  const y = window.scrollY;
  article.innerHTML = doc.html || "<p>Empty file.</p>";
  if (follow && nearBottom) {
    window.scrollTo(0, document.body.scrollHeight);
  } else {
    window.scrollTo(0, y);
  }
  document.title = doc.title || "mdread";
}

function flashLive() {
  const live = $("live");
  live.hidden = false;
  clearTimeout(flashLive._t);
  flashLive._t = setTimeout(() => {
    live.hidden = true;
  }, 1200);
}

async function persist() {
  const tauri = api();
  if (!tauri) return;
  await tauri.core.invoke("save_settings", { settings: state });
}

function bind() {
  $("gear").addEventListener("click", () => {
    $("panel").hidden = !$("panel").hidden;
  });

  $("font").addEventListener("change", async () => {
    if ($("font").value === "custom") {
      $("custom-font-wrap").hidden = false;
      $("custom-font").focus();
      return;
    }
    state.font = $("font").value;
    applyAppearance();
    await persist();
  });

  $("custom-font").addEventListener("change", async () => {
    const value = $("custom-font").value.trim();
    if (!value) return;
    state.font = value;
    applyAppearance();
    await persist();
  });

  $("size").addEventListener("input", () => {
    state.size = Number($("size").value);
    applyAppearance();
  });
  $("size").addEventListener("change", persist);

  $("decorations").addEventListener("change", async () => {
    state.decorations = $("decorations").value;
    await persist();
  });

  for (const button of document.querySelectorAll("[data-theme]")) {
    button.addEventListener("click", async () => {
      state.theme = button.dataset.theme;
      if (state.theme !== "custom" && PRESETS[state.theme]) {
        state.bg = PRESETS[state.theme].bg;
        state.fg = PRESETS[state.theme].fg;
      }
      applyAppearance();
      await persist();
    });
  }

  $("bg").addEventListener("input", () => {
    state.theme = "custom";
    state.bg = $("bg").value;
    applyAppearance();
  });
  $("fg").addEventListener("input", () => {
    state.theme = "custom";
    state.fg = $("fg").value;
    applyAppearance();
  });
  $("bg").addEventListener("change", persist);
  $("fg").addEventListener("change", persist);

  window.addEventListener("keydown", async (event) => {
    if (event.key === "Escape") {
      if (!$("panel").hidden) {
        $("panel").hidden = true;
        return;
      }
      api()?.window.getCurrentWindow().close();
    }
    if (event.key === "+" || event.key === "=") {
      state.size = Math.min(28, state.size + 1);
      applyAppearance();
      await persist();
    }
    if (event.key === "-" || event.key === "_") {
      state.size = Math.max(14, state.size - 1);
      applyAppearance();
      await persist();
    }
    if (event.key === "j") window.scrollBy(0, 40);
    if (event.key === "k") window.scrollBy(0, -40);
  });
}

async function main() {
  bind();
  applyAppearance();

  const tauri = api();
  if (!tauri) {
    for (let i = 0; i < 40 && !api(); i += 1) {
      await new Promise((resolve) => setTimeout(resolve, 50));
    }
  }
  const ready = api();
  if (!ready) {
    $("content").textContent = "mdread UI loaded without Tauri IPC.";
    return;
  }

  try {
    const boot = await ready.core.invoke("get_boot");
    Object.assign(state, boot.settings);
    applyAppearance();
    renderDocument(boot.document, false);
  } catch (error) {
    $("content").textContent = `Failed to load document: ${error}`;
    return;
  }

  await ready.event.listen("document-updated", (event) => {
    renderDocument(event.payload, true);
    flashLive();
  });
}

main().catch((error) => {
  const content = $("content");
  if (content) content.textContent = `Failed to start: ${error}`;
});
