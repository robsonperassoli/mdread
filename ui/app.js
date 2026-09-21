const PRESETS = {
  light: { bg: "#ffffff", fg: "#1f2328" },
  dark: { bg: "#0d1117", fg: "#f0f6fc" },
  sepia: { bg: "#f4ecd8", fg: "#5b4636" },
};

const state = {
  font: 'Georgia, "Iowan Old Style", Palatino, serif',
  size: 18,
  theme: "dark",
  bg: PRESETS.dark.bg,
  fg: PRESETS.dark.fg,
  decorations: "auto",
};

let fonts = [];

const $ = (id) => document.getElementById(id);

function api() {
  return window.__TAURI__;
}

function resolvedScheme() {
  if (state.theme === "light" || state.theme === "sepia") return "light";
  if (state.theme === "dark") return "dark";
  return luminance(state.bg) > 382 ? "light" : "dark";
}

function luminance(color) {
  const hex = String(color || "").replace("#", "");
  if (!/^[0-9a-fA-F]{6}$/.test(hex)) return 0;
  return (
    parseInt(hex.slice(0, 2), 16) + parseInt(hex.slice(2, 4), 16) + parseInt(hex.slice(4, 6), 16)
  );
}

function quoteFamily(name) {
  return `"${String(name).replaceAll('"', "")}"`;
}

function appliedFont() {
  if (!state.font) return "sans-serif";
  if (state.font.includes(",")) return state.font;
  return `${quoteFamily(state.font)}, sans-serif`;
}

function fontChoices() {
  const choices = fonts.map((family) => ({ value: family, label: family }));
  if (state.font && !fonts.includes(state.font)) {
    choices.unshift({ value: state.font, label: state.font });
  }
  return choices;
}

function applyAppearance() {
  const root = document.documentElement;
  const scheme = resolvedScheme();
  root.dataset.theme = state.theme || "dark";
  root.dataset.scheme = scheme;
  root.style.setProperty("--md-font", appliedFont());
  root.style.setProperty("--md-size", `${state.size}px`);
  root.style.setProperty("--md-bg", state.bg);
  root.style.setProperty("--md-fg", state.fg);

  $("gh-light").disabled = scheme !== "light";
  $("gh-dark").disabled = scheme === "light";
  if (state.theme === "sepia") {
    $("gh-light").disabled = false;
    $("gh-dark").disabled = true;
  }

  $("size").value = String(state.size);
  $("font-current").textContent = state.font || "Font";
  if (!$("font-menu").hidden) renderFontList($("font-filter").value);

  for (const button of document.querySelectorAll(".row [data-theme]")) {
    const preset = PRESETS[button.dataset.theme];
    const pressed =
      state.theme === button.dataset.theme &&
      preset &&
      state.bg.toLowerCase() === preset.bg &&
      state.fg.toLowerCase() === preset.fg;
    button.setAttribute("aria-pressed", String(pressed));
  }
}

function renderFontList(filter = "") {
  const list = $("font-list");
  const query = filter.trim().toLowerCase();
  list.replaceChildren();
  const matches = fontChoices().filter((choice) => choice.label.toLowerCase().includes(query));
  if (matches.length === 0) {
    const empty = document.createElement("li");
    empty.className = "empty";
    empty.textContent = "No matching fonts";
    list.append(empty);
    return;
  }
  for (const choice of matches) {
    const item = document.createElement("li");
    item.setAttribute("role", "option");
    item.dataset.value = choice.value;
    item.setAttribute("aria-selected", String(choice.value === state.font));
    item.textContent = choice.label;
    item.style.fontFamily = choice.value.includes(",")
      ? choice.value
      : `${quoteFamily(choice.value)}, sans-serif`;
    list.append(item);
  }
  list.querySelector('[aria-selected="true"]')?.scrollIntoView({ block: "nearest" });
}

function openFonts() {
  $("font-menu").hidden = false;
  $("font-button").setAttribute("aria-expanded", "true");
  $("font-filter").value = "";
  renderFontList();
  $("font-filter").focus();
}

function closeFonts() {
  $("font-menu").hidden = true;
  $("font-button").setAttribute("aria-expanded", "false");
}

function renderDocument(doc, follow) {
  const article = $("content");
  const nearBottom = window.innerHeight + window.scrollY >= document.body.scrollHeight - 96;
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
    const opening = $("panel").hidden;
    $("panel").hidden = !opening;
    if (!opening) closeFonts();
  });

  $("font-button").addEventListener("click", () => {
    if ($("font-menu").hidden) openFonts();
    else closeFonts();
  });

  $("font-filter").addEventListener("input", () => {
    renderFontList($("font-filter").value);
  });

  $("font-list").addEventListener("click", async (event) => {
    const item = event.target.closest("[data-value]");
    if (!item) return;
    state.font = item.dataset.value;
    closeFonts();
    applyAppearance();
    await persist();
  });

  document.addEventListener("pointerdown", (event) => {
    if ($("font-menu").hidden) return;
    if ($("font-picker").contains(event.target)) return;
    closeFonts();
  });

  $("size").addEventListener("input", () => {
    state.size = Number($("size").value);
    applyAppearance();
  });
  $("size").addEventListener("change", persist);

  for (const button of document.querySelectorAll(".row [data-theme]")) {
    button.addEventListener("click", async () => {
      state.theme = button.dataset.theme;
      const preset = PRESETS[state.theme];
      if (preset) {
        state.bg = preset.bg;
        state.fg = preset.fg;
      }
      applyAppearance();
      await persist();
    });
  }

  window.addEventListener("keydown", async (event) => {
    const typing = event.target.closest && event.target.closest("input, textarea");
    if (event.key === "Escape") {
      if (!$("font-menu").hidden) {
        closeFonts();
        $("font-button").focus();
        return;
      }
      if (!$("panel").hidden) {
        $("panel").hidden = true;
        return;
      }
      api()?.window.getCurrentWindow().close();
      return;
    }
    if (typing) return;
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

  await ready.event.listen("document-updated", (event) => {
    renderDocument(event.payload, true);
    flashLive();
  });
  await ready.event.listen("settings-updated", (event) => {
    Object.assign(state, event.payload);
    applyAppearance();
  });

  try {
    const boot = await ready.core.invoke("get_boot");
    Object.assign(state, boot.settings);
    fonts = Array.isArray(boot.fonts) ? boot.fonts : [];
    applyAppearance();
    renderDocument(boot.document, false);
  } catch (error) {
    $("content").textContent = `Failed to load document: ${error}`;
  }
}

main().catch((error) => {
  const content = $("content");
  if (content) content.textContent = `Failed to start: ${error}`;
});
