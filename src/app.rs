use std::path::PathBuf;
use std::time::Instant;

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::color_picker::{ColorPicker, ColorPickerEvent, ColorPickerState};
use gpui_kit::component::slider::{Slider, SliderEvent, SliderState};
use gpui_kit::component::text::{TextView, TextViewState};
use gpui_kit::component::{h_flex, v_flex, ActiveTheme as _, Root, Selectable, TitleBar};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    div, px, rems, App, AppContext as _, Bounds, Context, Entity, FocusHandle, InteractiveElement,
    IntoElement, KeyBinding, ParentElement, Render, ScrollHandle, SharedString, Size,
    StatefulInteractiveElement, Styled, Subscription, TitlebarOptions, Window, WindowBounds,
    WindowDecorations,
};

use crate::appearance;
use crate::color;
use crate::config::{self, Settings, CUSTOM_THEME};
use crate::markdown;
use crate::watch::{self, WatchEvent};

gpui_kit::actions!(
    mdread,
    [
        Quit,
        TogglePanel,
        ScrollDown,
        ScrollUp,
        IncreaseSize,
        DecreaseSize
    ]
);

const FOLLOW_THRESHOLD: f32 = 96.0;

pub fn init(cx: &mut App) {
    appearance::register_bundled_themes(cx);
    cx.bind_keys([
        KeyBinding::new("escape", Quit, Some("mdread")),
        KeyBinding::new("j", ScrollDown, Some("mdread")),
        KeyBinding::new("k", ScrollUp, Some("mdread")),
        KeyBinding::new("=", IncreaseSize, Some("mdread")),
        KeyBinding::new("-", DecreaseSize, Some("mdread")),
        KeyBinding::new(",", TogglePanel, Some("mdread")),
    ]);
}

pub fn open(file: PathBuf, settings: Settings, cx: &mut App) {
    let title = file
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("mdread")
        .to_string();
    let hide_title_bar = settings.decorations.hide_title_bar();
    let bounds = Bounds::centered(None, Size::new(px(880.), px(960.)), cx);
    let mut options = TitleBar::window_options();
    options.window_bounds = Some(WindowBounds::Windowed(bounds));
    options.window_min_size = Some(Size::new(px(480.), px(360.)));
    options.app_id = Some("mdread".into());
    options.titlebar = Some(TitlebarOptions {
        title: Some(title.clone().into()),
        ..TitleBar::title_bar_options()
    });
    #[cfg(target_os = "linux")]
    {
        options.window_decorations = Some(WindowDecorations::Client);
    }

    let file_for_watch = file.clone();
    cx.spawn(async move |cx| {
        let opened = cx.open_window(options, |window, cx| {
            let reader = cx.new(|cx| Reader::new(file, settings, hide_title_bar, window, cx));
            start_watchers(file_for_watch, reader.downgrade(), cx);
            cx.new(|cx| Root::new(reader, window, cx))
        });
        if let Err(err) = opened {
            eprintln!("mdread: failed to open window: {err}");
        }
    })
    .detach();
}

fn start_watchers(file: PathBuf, reader: gpui_kit::WeakEntity<Reader>, cx: &mut App) {
    let (tx, rx) = smol::channel::unbounded();
    watch::spawn(file, tx);
    cx.spawn(async move |cx| {
        while let Ok(event) = rx.recv().await {
            let alive = cx.update(|cx| {
                if let Some(entity) = reader.upgrade() {
                    entity.update(cx, |this, cx| this.on_watch(event, cx));
                    true
                } else {
                    false
                }
            });
            if !alive {
                break;
            }
        }
    })
    .detach();
}

struct Reader {
    file: PathBuf,
    settings: Settings,
    document: Entity<TextViewState>,
    scroll: ScrollHandle,
    panel_open: bool,
    hide_title_bar: bool,
    fonts: Vec<String>,
    theme_names: Vec<SharedString>,
    size_slider: Entity<SliderState>,
    radius_slider: Entity<SliderState>,
    bg_picker: Entity<ColorPickerState>,
    fg_picker: Entity<ColorPickerState>,
    muted_picker: Entity<ColorPickerState>,
    border_picker: Entity<ColorPickerState>,
    primary_picker: Entity<ColorPickerState>,
    focus: FocusHandle,
    flash_until: Option<Instant>,
    _subscriptions: Vec<Subscription>,
}

impl Reader {
    fn new(
        file: PathBuf,
        settings: Settings,
        hide_title_bar: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        appearance::apply_settings(&settings, Some(window), cx);
        let source = markdown::load_source(&file);
        let document = cx.new(|cx| TextViewState::markdown(&source, cx).selectable(true));
        let size_slider = cx.new(|_| {
            SliderState::new()
                .min(14.)
                .max(28.)
                .step(1.)
                .default_value(settings.size)
        });
        let radius_slider = cx.new(|_| {
            SliderState::new()
                .min(0.)
                .max(16.)
                .step(1.)
                .default_value(settings.radius)
        });

        let palette = &settings.custom;
        let bg_picker = color_picker(window, cx, &palette.background);
        let fg_picker = color_picker(window, cx, &palette.foreground);
        let muted_picker = color_picker(window, cx, palette.muted.as_deref().unwrap_or("#8b949e"));
        let border_picker =
            color_picker(window, cx, palette.border.as_deref().unwrap_or("#30363d"));
        let primary_picker =
            color_picker(window, cx, palette.primary.as_deref().unwrap_or("#4493f8"));

        let mut subscriptions = Vec::new();
        subscriptions.push(
            cx.subscribe(&size_slider, |this, slider, _: &SliderEvent, cx| {
                let value = slider.read(cx).value().end();
                if (this.settings.size - value).abs() < f32::EPSILON {
                    return;
                }
                this.settings.size = value;
                this.persist_and_apply(cx);
            }),
        );
        subscriptions.push(
            cx.subscribe(&radius_slider, |this, slider, _: &SliderEvent, cx| {
                let value = slider.read(cx).value().end();
                if (this.settings.radius - value).abs() < f32::EPSILON {
                    return;
                }
                this.settings.radius = value;
                this.settings.radius_lg = (value + 2.0).max(value);
                this.persist_and_apply(cx);
            }),
        );
        subscriptions.push(subscribe_color(cx, &bg_picker, |settings, hex| {
            settings.custom.background = hex
        }));
        subscriptions.push(subscribe_color(cx, &fg_picker, |settings, hex| {
            settings.custom.foreground = hex
        }));
        subscriptions.push(subscribe_color(cx, &muted_picker, |settings, hex| {
            settings.custom.muted = Some(hex)
        }));
        subscriptions.push(subscribe_color(cx, &border_picker, |settings, hex| {
            settings.custom.border = Some(hex)
        }));
        subscriptions.push(subscribe_color(cx, &primary_picker, |settings, hex| {
            settings.custom.primary = Some(hex)
        }));

        let focus = cx.focus_handle();
        focus.focus(window, cx);

        Self {
            file,
            settings,
            document,
            scroll: ScrollHandle::new(),
            panel_open: false,
            hide_title_bar,
            fonts: crate::fonts::installed(),
            theme_names: appearance::theme_names(cx),
            size_slider,
            radius_slider,
            bg_picker,
            fg_picker,
            muted_picker,
            border_picker,
            primary_picker,
            focus,
            flash_until: None,
            _subscriptions: subscriptions,
        }
    }

    fn persist_and_apply(&mut self, cx: &mut Context<Self>) {
        if let Err(err) = config::save(&self.settings) {
            eprintln!("mdread: could not save settings: {err}");
        }
        appearance::apply_settings(&self.settings, None, cx);
        cx.notify();
    }

    fn on_watch(&mut self, event: WatchEvent, cx: &mut Context<Self>) {
        match event {
            WatchEvent::File => self.reload_document(true, cx),
            WatchEvent::Config => {
                let loaded = config::load();
                if loaded == self.settings {
                    return;
                }
                self.settings = loaded;
                self.hide_title_bar = self.settings.decorations.hide_title_bar();
                self.sync_controls(cx);
                appearance::apply_settings(&self.settings, None, cx);
                cx.notify();
            }
        }
    }

    fn reload_document(&mut self, follow: bool, cx: &mut Context<Self>) {
        let near_bottom = self.near_bottom();
        let source = markdown::load_source(&self.file);
        self.document
            .update(cx, |state, cx| state.set_text(&source, cx));
        if follow && near_bottom {
            self.scroll.scroll_to_bottom();
        }
        self.flash_until = Some(Instant::now() + std::time::Duration::from_millis(1200));
        cx.notify();
    }

    fn near_bottom(&self) -> bool {
        let offset = self.scroll.offset();
        let max = self.scroll.max_offset();
        (max.y + offset.y).abs() <= px(FOLLOW_THRESHOLD)
    }

    fn sync_controls(&mut self, _cx: &mut Context<Self>) {}

    fn set_theme(&mut self, theme: impl Into<String>, cx: &mut Context<Self>) {
        self.settings.theme = theme.into();
        self.persist_and_apply(cx);
    }

    fn set_font(&mut self, font: Option<String>, cx: &mut Context<Self>) {
        self.settings.font = font;
        self.persist_and_apply(cx);
    }

    fn bump_size(&mut self, delta: f32, window: &mut Window, cx: &mut Context<Self>) {
        self.settings.size = (self.settings.size + delta).clamp(14.0, 28.0);
        self.size_slider.update(cx, |slider, cx| {
            slider.set_value(self.settings.size, window, cx);
        });
        self.persist_and_apply(cx);
    }

    fn on_quit(&mut self, _: &Quit, window: &mut Window, cx: &mut Context<Self>) {
        if self.panel_open {
            self.panel_open = false;
            cx.notify();
            return;
        }
        window.remove_window();
    }

    fn on_toggle_panel(&mut self, _: &TogglePanel, _: &mut Window, cx: &mut Context<Self>) {
        self.panel_open = !self.panel_open;
        cx.notify();
    }

    fn on_scroll_down(&mut self, _: &ScrollDown, _: &mut Window, _: &mut Context<Self>) {
        let mut offset = self.scroll.offset();
        offset.y -= px(40.);
        self.scroll.set_offset(offset);
    }

    fn on_scroll_up(&mut self, _: &ScrollUp, _: &mut Window, _: &mut Context<Self>) {
        let mut offset = self.scroll.offset();
        offset.y += px(40.);
        self.scroll.set_offset(offset);
    }

    fn on_increase(&mut self, _: &IncreaseSize, window: &mut Window, cx: &mut Context<Self>) {
        self.bump_size(1.0, window, cx);
    }

    fn on_decrease(&mut self, _: &DecreaseSize, window: &mut Window, cx: &mut Context<Self>) {
        self.bump_size(-1.0, window, cx);
    }

    fn render_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let current_theme = self.settings.theme.clone();
        let current_font = self.settings.font.clone();
        let is_custom = self.settings.is_custom();
        let custom_mode = self.settings.custom.mode.clone();

        v_flex()
            .w(px(280.))
            .flex_1()
            .p_3()
            .gap_3()
            .border_l_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().popover)
            .text_sm()
            .child(
                div()
                    .text_sm()
                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                    .child("Reading"),
            )
            .child(label("Theme", cx))
            .child(
                div()
                    .id("themes")
                    .max_h(px(180.))
                    .overflow_y_scroll()
                    .w_full()
                    .child(
                        v_flex().gap_1().children(
                            std::iter::once(CUSTOM_THEME.to_string())
                                .chain(self.theme_names.iter().map(|name| name.to_string()))
                                .map(|name| {
                                    let selected = current_theme == name;
                                    let label = name.clone();
                                    Button::new(SharedString::from(format!("theme-{name}")))
                                        .w_full()
                                        .ghost()
                                        .selected(selected)
                                        .label(label)
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.set_theme(name.clone(), cx);
                                        }))
                                }),
                        ),
                    ),
            )
            .child(label("Font", cx))
            .child(
                div()
                    .id("fonts")
                    .max_h(px(160.))
                    .overflow_y_scroll()
                    .w_full()
                    .child(
                        v_flex()
                            .gap_1()
                            .child(
                                Button::new("font-system")
                                    .w_full()
                                    .ghost()
                                    .selected(current_font.is_none())
                                    .label("System")
                                    .on_click(
                                        cx.listener(|this, _, _, cx| this.set_font(None, cx)),
                                    ),
                            )
                            .children(self.fonts.iter().cloned().map(|family| {
                                let selected = current_font.as_deref() == Some(family.as_str());
                                Button::new(SharedString::from(format!("font-{family}")))
                                    .w_full()
                                    .ghost()
                                    .selected(selected)
                                    .label(family.clone())
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.set_font(Some(family.clone()), cx);
                                    }))
                            })),
                    ),
            )
            .child(label(format!("Size {}", self.settings.size as i32), cx))
            .child(Slider::new(&self.size_slider).horizontal())
            .child(label(format!("Radius {}", self.settings.radius as i32), cx))
            .child(Slider::new(&self.radius_slider).horizontal())
            .child(
                Button::new("shadow")
                    .ghost()
                    .selected(self.settings.shadow)
                    .label(if self.settings.shadow {
                        "Shadows on"
                    } else {
                        "Shadows off"
                    })
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.settings.shadow = !this.settings.shadow;
                        this.persist_and_apply(cx);
                    })),
            )
            .when(is_custom, |this| {
                this.child(label("Custom colors", cx))
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("mode-dark")
                                    .ghost()
                                    .selected(custom_mode == "dark")
                                    .label("Dark")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.settings.custom.mode = "dark".into();
                                        this.persist_and_apply(cx);
                                    })),
                            )
                            .child(
                                Button::new("mode-light")
                                    .ghost()
                                    .selected(custom_mode == "light")
                                    .label("Light")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.settings.custom.mode = "light".into();
                                        this.persist_and_apply(cx);
                                    })),
                            ),
                    )
                    .child(ColorPicker::new(&self.bg_picker).label("Background"))
                    .child(ColorPicker::new(&self.fg_picker).label("Foreground"))
                    .child(ColorPicker::new(&self.muted_picker).label("Muted"))
                    .child(ColorPicker::new(&self.border_picker).label("Border"))
                    .child(ColorPicker::new(&self.primary_picker).label("Primary"))
            })
    }
}

fn label(text: impl Into<SharedString>, cx: &App) -> impl IntoElement {
    div()
        .text_xs()
        .text_color(cx.theme().muted_foreground)
        .child(text.into())
}

fn color_picker(
    window: &mut Window,
    cx: &mut Context<Reader>,
    hex: &str,
) -> Entity<ColorPickerState> {
    let value = color::parse_hex(hex).unwrap_or_else(|| gpui_kit::hsla(0., 0., 0.2, 1.));
    cx.new(|cx| ColorPickerState::new(window, cx).default_value(value))
}

fn subscribe_color(
    cx: &mut Context<Reader>,
    picker: &Entity<ColorPickerState>,
    write: fn(&mut Settings, String),
) -> Subscription {
    cx.subscribe(picker, move |this, _, event: &ColorPickerEvent, cx| {
        let ColorPickerEvent::Change(Some(value)) = event else {
            return;
        };
        write(&mut this.settings, color::to_hex(*value));
        if !this.settings.is_custom() {
            this.settings.theme = CUSTOM_THEME.into();
        }
        this.persist_and_apply(cx);
    })
}

impl Render for Reader {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let flashing = self.flash_until.is_some_and(|until| Instant::now() < until);
        let title = self
            .file
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("mdread")
            .to_string();

        div()
            .id("mdread")
            .key_context("mdread")
            .track_focus(&self.focus)
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .font_family(cx.theme().font_family.clone())
            .text_size(cx.theme().font_size)
            .on_action(cx.listener(Self::on_quit))
            .on_action(cx.listener(Self::on_toggle_panel))
            .on_action(cx.listener(Self::on_scroll_down))
            .on_action(cx.listener(Self::on_scroll_up))
            .on_action(cx.listener(Self::on_increase))
            .on_action(cx.listener(Self::on_decrease))
            .child(
                v_flex()
                    .size_full()
                    .when(!self.hide_title_bar, |this| {
                        this.child(TitleBar::new().child(div().text_sm().child(title.clone())))
                    })
                    .child(
                        div()
                            .id("body")
                            .flex()
                            .flex_1()
                            .min_h_0()
                            .child(
                                div()
                                    .id("document")
                                    .flex_1()
                                    .min_w_0()
                                    .relative()
                                    .child(
                                        div()
                                            .id("scroll")
                                            .size_full()
                                            .overflow_y_scroll()
                                            .track_scroll(&self.scroll)
                                            .child(
                                                div()
                                                    .id("article")
                                                    .max_w(rems(45.))
                                                    .mx_auto()
                                                    .px_6()
                                                    .py_8()
                                                    .child(TextView::new(&self.document)),
                                            ),
                                    )
                                    .child(
                                        h_flex()
                                            .absolute()
                                            .top_3()
                                            .right_3()
                                            .gap_2()
                                            .items_center()
                                            .child(Button::new("aa").ghost().label("Aa").on_click(
                                                cx.listener(|this, _, _, cx| {
                                                    this.panel_open = !this.panel_open;
                                                    cx.notify();
                                                }),
                                            ))
                                            .when(flashing, |this| {
                                                this.child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(cx.theme().muted_foreground)
                                                        .child("Updated"),
                                                )
                                            }),
                                    ),
                            )
                            .when(self.panel_open, |this| this.child(self.render_panel(cx))),
                    ),
            )
    }
}
