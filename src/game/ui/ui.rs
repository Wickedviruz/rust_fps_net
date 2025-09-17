use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use crate::game::app_state::AppState;

use super::crosshair;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .insert_resource(MenuState::default())
            .add_systems(Startup, setup_egui_theme)
            .add_systems(Update, main_menu_ui.run_if(in_state(AppState::MainMenu)))
            .add_systems(Update, pause_menu_ui.run_if(in_state(AppState::Paused)))
            .add_systems(Update, crosshair::spawn_crosshair.run_if(in_state(AppState::InGame)))
            .add_systems(Update, toggle_pause);
    }
}

/* ---------------- State för menyn ---------------- */

#[derive(Resource, Default)]
struct MenuState {
    top_tab: TopTab,
    selected_news: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TopTab { Play, Inventory, Watch, Awards, Options }
impl Default for TopTab { fn default() -> Self { TopTab::Play } }

/* ---------------- Tema / Skin ---------------- */

fn setup_egui_theme(mut egui_ctx: EguiContexts) {
    let ctx = egui_ctx.ctx_mut();
    let mut style = (*ctx.style()).clone();

    style.spacing.item_spacing = egui::vec2(12.0, 10.0);
    style.spacing.window_margin = egui::Margin::same(14.0);

    // Basvisuals
    style.visuals = egui::Visuals {
        dark_mode: true,
        panel_fill: egui::Color32::from_black_alpha(60),
        window_rounding: egui::Rounding::same(6.0),
        // <-- Skugga i egui 0.28 sätts så här:
        window_shadow: egui::epaint::Shadow {
            offset: egui::vec2(0.0, 4.0),                  // skuggans förskjutning
            blur: 12.0,                                    // hur mjuk den är
            spread: 0.0,                                   // extra storlek
            color: egui::Color32::from_black_alpha(100),        // färg/alpha på skuggan
        },
        override_text_color: Some(egui::Color32::from_gray(225)),
        ..egui::Visuals::dark()
    };

    // (valfritt) runda hörn på widgets
    style.visuals.widgets.inactive.rounding = egui::Rounding::same(6.0);
    style.visuals.widgets.hovered.rounding  = egui::Rounding::same(6.0);
    style.visuals.widgets.active.rounding   = egui::Rounding::same(6.0);

    ctx.set_style(style);
}

fn frame_card() -> egui::Frame {
    egui::Frame::none()
        .fill(egui::Color32::from_black_alpha(100))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_white_alpha(28)))
        .rounding(6.0)
        .inner_margin(egui::Margin::same(10.0))
}

fn frame_topbar() -> egui::Frame {
    egui::Frame::none()
        .fill(egui::Color32::from_black_alpha(140))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_white_alpha(30)))
        .inner_margin(egui::Margin::symmetric(12.0, 6.0))
}

/* ---------------- Huvudmeny ---------------- */

fn main_menu_ui(
    mut egui_ctx: EguiContexts,
    mut menu: ResMut<MenuState>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let ctx = egui_ctx.ctx_mut();

    // TOP BAR
    egui::TopBottomPanel::top("topbar")
        .frame(frame_topbar())
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                top_tab_button(ui, &mut menu.top_tab, TopTab::Play, "PLAY");
                top_tab_button(ui, &mut menu.top_tab, TopTab::Inventory, "INVENTORY");
                top_tab_button(ui, &mut menu.top_tab, TopTab::Watch, "WATCH");
                top_tab_button(ui, &mut menu.top_tab, TopTab::Awards, "AWARDS");
                top_tab_button(ui, &mut menu.top_tab, TopTab::Options, "OPTIONS");
            });
        });

    // LEFT – profil & friends
    egui::SidePanel::left("left")
        .exact_width(320.0)
        .frame(egui::Frame::none())
        .show(ctx, |ui| {
            frame_card().show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.heading("PlayerName");
                        ui.label("Sergeant • Rank 12");
                        ui.label("Gold Nova I");
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        ui.label("LVL 27");
                    });
                });
            });
            ui.add_space(8.0);
            frame_card().show(ui, |ui| {
                ui.label(egui::RichText::new("FRIENDS").weak());
                ui.separator();
                egui::ScrollArea::vertical().max_height(420.0).show(ui, |ui| {
                    for i in 0..12 {
                        ui.horizontal(|ui| {
                            ui.small(format!("Friend_{}", i));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.weak("Online");
                            });
                        });
                        ui.separator();
                    }
                });
            });
        });

    // RIGHT – queue / info
    egui::SidePanel::right("right")
        .exact_width(300.0)
        .frame(egui::Frame::none())
        .show(ctx, |ui| {
            frame_card().show(ui, |ui| {
                ui.label(egui::RichText::new("YOUR QUEUE").weak());
                ui.separator();
                ui.label("Competitive • Mirage");
                ui.add_space(8.0);
                if ui.button("Find Match").clicked() && matches!(menu.top_tab, TopTab::Play) {
                    next_state.set(AppState::InGame);
                }
            });
            ui.add_space(8.0);
            frame_card().show(ui, |ui| {
                ui.label(egui::RichText::new("LIVE STREAMS").weak());
                ui.separator();
                for _ in 0..4 { ui.small("StreamerXYZ • 12.5k viewers"); }
            });
        });

    // CENTER – hero + news
    egui::CentralPanel::default()
        .frame(egui::Frame::none().inner_margin(egui::Margin::symmetric(12.0, 10.0)))
        .show(ctx, |ui| {
            // HERO / banner
            frame_card().show(ui, |ui| {
                ui.set_height(160.0);
                ui.vertical_centered(|ui| {
                    ui.heading("Operation Wildfire – New Missions Available");
                    ui.label("2 days 12:34:56 remaining");
                });
            });

            ui.add_space(10.0);

            // NEWS
            frame_card().show(ui, |ui| {
                ui.horizontal(|ui| {
                    tab(ui, &mut menu.selected_news, 0, "BLOG");
                    tab(ui, &mut menu.selected_news, 1, "UPDATES");
                    tab(ui, &mut menu.selected_news, 2, "RESOURCES");
                    tab(ui, &mut menu.selected_news, 3, "COMMUNITY");
                });
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for n in 0..6 {
                        ui.label(egui::RichText::new(format!("ESL One Cologne {} – News item", 2016 + n)).strong());
                        ui.small("CS:GO's Next Major • 8 APR 20xx");
                        ui.add_space(6.0);
                        ui.label("Teaser text lorem ipsum dolor sit amet, consectetur adipiscing elit. Integer vitae…");
                        ui.add_space(10.0);
                        ui.separator();
                    }
                });
            });
        });
}

/* ---------------- Pausmeny (overlay) ---------------- */

fn pause_menu_ui(mut egui_ctx: EguiContexts, mut next_state: ResMut<NextState<AppState>>) {
    let ctx = egui_ctx.ctx_mut();

    // Mörk tint
    egui::Area::new(egui::Id::new("pause_tint"))
        .interactable(false)
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            ui.painter().rect_filled(rect, 0.0, egui::Color32::from_black_alpha(180));
        });

    // Panel
    egui::Window::new("Paused")
        .collapsible(false)
        .resizable(false)
        .frame(frame_card())
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            if ui.button("Resume").clicked() { next_state.set(AppState::InGame); }
            if ui.button("Main Menu").clicked() { next_state.set(AppState::MainMenu); }
            if ui.button("Quit").clicked() { std::process::exit(0); }
        });
}

/* ---------------- Input: ESC för pausa ---------------- */

fn toggle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        match state.get() {
            AppState::InGame => next_state.set(AppState::Paused),
            AppState::Paused => next_state.set(AppState::InGame),
            _ => {}
        }
    }
}

/* ---------------- Små helpers ---------------- */

fn top_tab_button(ui: &mut egui::Ui, current: &mut TopTab, tab: TopTab, label: &str) {
    let selected = *current == tab;
    let (bg, stroke) = if selected {
        (egui::Color32::from_black_alpha(80), egui::Stroke::new(1.0, egui::Color32::WHITE))
    } else {
        (egui::Color32::from_black_alpha(40), egui::Stroke::new(1.0, egui::Color32::from_white_alpha(40)))
    };
    let frame = egui::Frame::none().fill(bg).stroke(stroke).rounding(6.0).inner_margin(egui::Margin::symmetric(12.0, 6.0));
    frame.show(ui, |ui| {
        if ui.selectable_label(selected, label).clicked() {
            *current = tab;
        }
    });
}

fn tab(ui: &mut egui::Ui, sel: &mut usize, idx: usize, text: &str) {
    let selected = *sel == idx;
    let rt = if selected { egui::RichText::new(text).strong() } else { egui::RichText::new(text).weak() };
    if ui.selectable_label(selected, rt).clicked() {
        *sel = idx;
    }
}
