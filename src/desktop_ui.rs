use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use eframe::egui::{self, Color32, FontData, FontDefinitions, FontFamily, RichText, Stroke};
use study_rpg::{Dashboard, DesktopController};

const BACKGROUND: Color32 = Color32::from_rgb(12, 16, 28);
const PANEL: Color32 = Color32::from_rgb(24, 30, 48);
const PANEL_HOVER: Color32 = Color32::from_rgb(31, 39, 61);
const ACCENT: Color32 = Color32::from_rgb(250, 190, 72);
const SUCCESS: Color32 = Color32::from_rgb(91, 214, 145);
const DANGER: Color32 = Color32::from_rgb(244, 112, 128);
const MUTED: Color32 = Color32::from_rgb(155, 166, 190);

pub struct StudyRpgDesktopApp {
    controller: DesktopController,
    topic: String,
    feedback: Option<Feedback>,
}

struct Feedback {
    message: String,
    is_error: bool,
}

impl StudyRpgDesktopApp {
    pub fn new(
        controller: DesktopController,
        creation_context: &eframe::CreationContext<'_>,
    ) -> Self {
        configure_style(&creation_context.egui_ctx);
        install_system_cjk_font(&creation_context.egui_ctx);

        Self {
            controller,
            topic: String::new(),
            feedback: None,
        }
    }

    fn render_header(ui: &mut egui::Ui, dashboard: &Dashboard) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("STUDY RPG").color(ACCENT).strong().size(14.0));
                ui.label(
                    RichText::new(format!("Welcome back, {}", dashboard.player_name))
                        .strong()
                        .size(30.0),
                );
                ui.label(RichText::new(&dashboard.title).color(MUTED).size(15.0));
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                egui::Frame::new()
                    .fill(PANEL)
                    .corner_radius(12.0)
                    .inner_margin(14.0)
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(format!("LV. {}", dashboard.level.level))
                                .color(ACCENT)
                                .strong()
                                .size(22.0),
                        );
                    });
            });
        });

        ui.add_space(12.0);
        ui.add(
            egui::ProgressBar::new(f32::from(dashboard.xp_progress_percent) / 100.0)
                .desired_width(ui.available_width())
                .fill(ACCENT)
                .text(format!(
                    "{} / {} XP to next level",
                    dashboard.level.xp_into_level, dashboard.level.xp_for_next_level
                )),
        );
    }

    fn render_stat_cards(ui: &mut egui::Ui, dashboard: &Dashboard) {
        ui.columns(3, |columns| {
            stat_card(&mut columns[0], "TODAY", dashboard.today_minutes, "minutes");
            stat_card(
                &mut columns[1],
                "SESSIONS",
                dashboard.total_sessions,
                "completed",
            );
            stat_card(&mut columns[2], "TOTAL XP", dashboard.total_xp, "earned");
        });
    }

    fn render_session_panel(&mut self, ui: &mut egui::Ui, dashboard: &Dashboard, now: u64) {
        section_frame(ui, |ui| {
            ui.label(RichText::new("FOCUS SESSION").color(MUTED).strong());
            ui.add_space(8.0);

            if let Some(active) = &dashboard.active_session {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new(&active.topic).strong().size(22.0));
                        ui.label(
                            RichText::new(format!(
                                "{} min elapsed · estimated +{} XP",
                                active.elapsed_minutes, active.estimated_xp
                            ))
                            .color(MUTED),
                        );
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(RichText::new("Finish Session").strong())
                                    .fill(DANGER),
                            )
                            .clicked()
                        {
                            match self.controller.finish_session(now) {
                                Ok(result) => {
                                    self.feedback = Some(Feedback {
                                        message: session_completion_message(
                                            result.session.duration_minutes,
                                            result.session.earned_xp,
                                            result.quest_reward_xp,
                                            result.daily_completion_bonus_xp,
                                        ),
                                        is_error: false,
                                    });
                                }
                                Err(error) => self.set_error(error.to_string()),
                            }
                        }
                    });
                });
            } else {
                ui.horizontal(|ui| {
                    let input = ui.add_sized(
                        [ui.available_width() - 130.0, 38.0],
                        egui::TextEdit::singleline(&mut self.topic)
                            .hint_text("What are you studying?"),
                    );
                    let enter_pressed =
                        input.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
                    let start_clicked = ui
                        .add_sized(
                            [118.0, 38.0],
                            egui::Button::new(RichText::new("Start Study").strong()).fill(ACCENT),
                        )
                        .clicked();

                    if enter_pressed || start_clicked {
                        match self.controller.start_session(&self.topic, now) {
                            Ok(()) => {
                                let topic = self.topic.trim().to_string();
                                self.topic.clear();
                                self.feedback = Some(Feedback {
                                    message: format!("Timer started for {topic}"),
                                    is_error: false,
                                });
                            }
                            Err(error) => self.set_error(error.to_string()),
                        }
                    }
                });
            }
        });
    }

    fn render_feedback(&self, ui: &mut egui::Ui) {
        if let Some(feedback) = &self.feedback {
            let color = if feedback.is_error { DANGER } else { SUCCESS };
            egui::Frame::new()
                .fill(color.gamma_multiply(0.12))
                .stroke(Stroke::new(1.0, color.gamma_multiply(0.65)))
                .corner_radius(8.0)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.label(RichText::new(&feedback.message).color(color));
                });
        }
    }

    fn render_quests(ui: &mut egui::Ui, dashboard: &Dashboard) {
        section_frame(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("DAILY QUESTS").color(MUTED).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!(
                            "Daily clear +{} XP",
                            dashboard.daily_quest_completion.reward_xp
                        ))
                        .color(ACCENT),
                    );
                });
            });
            ui.add_space(8.0);

            for quest in &dashboard.quest_progress {
                ui.horizontal(|ui| {
                    let marker = if quest.completed { "✓" } else { "○" };
                    let marker_color = if quest.completed { SUCCESS } else { MUTED };
                    ui.label(
                        RichText::new(marker)
                            .color(marker_color)
                            .strong()
                            .size(18.0),
                    );
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&quest.title).strong());
                            ui.label(
                                RichText::new(format!("+{} XP", quest.reward_xp)).color(ACCENT),
                            );
                        });
                        let progress = if quest.target == 0 {
                            1.0
                        } else {
                            quest.current as f32 / quest.target as f32
                        };
                        ui.add(
                            egui::ProgressBar::new(progress)
                                .desired_width(ui.available_width())
                                .fill(if quest.completed { SUCCESS } else { ACCENT })
                                .text(format!("{} / {}", quest.current, quest.target)),
                        );
                    });
                });
                ui.add_space(8.0);
            }

            if dashboard.daily_quest_completion.completed {
                ui.label(
                    RichText::new(format!(
                        "Daily Complete · +{} XP claimed",
                        dashboard.daily_quest_completion.reward_xp
                    ))
                    .color(SUCCESS)
                    .strong(),
                );
            }
        });
    }

    fn render_recent_sessions(ui: &mut egui::Ui, dashboard: &Dashboard) {
        section_frame(ui, |ui| {
            ui.label(RichText::new("RECENT STUDY").color(MUTED).strong());
            ui.add_space(8.0);

            if dashboard.recent_sessions.is_empty() {
                ui.label(
                    RichText::new("Complete your first session to begin the journey.").color(MUTED),
                );
                return;
            }

            for session in &dashboard.recent_sessions {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new(&session.topic).strong());
                        if let Some(skill_name) = &session.skill_name {
                            ui.label(RichText::new(skill_name).color(MUTED).small());
                        }
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(format!("+{} XP", session.earned_xp)).color(ACCENT));
                        ui.label(
                            RichText::new(format!("{} min", session.duration_minutes)).color(MUTED),
                        );
                    });
                });
                ui.separator();
            }
        });
    }

    fn set_error(&mut self, message: String) {
        self.feedback = Some(Feedback {
            message,
            is_error: true,
        });
    }
}

impl eframe::App for StudyRpgDesktopApp {
    fn logic(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        context.request_repaint_after(Duration::from_secs(1));
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let now = current_epoch_seconds();
        let dashboard = match self.controller.dashboard_at(now) {
            Ok(dashboard) => dashboard,
            Err(error) => {
                egui::CentralPanel::default()
                    .frame(egui::Frame::new().fill(BACKGROUND).inner_margin(24.0))
                    .show(ui, |ui| {
                        ui.heading("Study RPG could not load local data");
                        ui.label(RichText::new(error.to_string()).color(DANGER));
                    });
                return;
            }
        };

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(BACKGROUND).inner_margin(24.0))
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    Self::render_header(ui, &dashboard);
                    ui.add_space(18.0);
                    Self::render_stat_cards(ui, &dashboard);
                    ui.add_space(12.0);
                    self.render_session_panel(ui, &dashboard, now);
                    if self.feedback.is_some() {
                        ui.add_space(10.0);
                        self.render_feedback(ui);
                    }
                    ui.add_space(12.0);
                    ui.columns(2, |columns| {
                        Self::render_quests(&mut columns[0], &dashboard);
                        Self::render_recent_sessions(&mut columns[1], &dashboard);
                    });
                });
            });
    }
}

fn stat_card(ui: &mut egui::Ui, label: &str, value: u32, suffix: &str) {
    egui::Frame::new()
        .fill(PANEL)
        .stroke(Stroke::new(1.0, PANEL_HOVER))
        .corner_radius(10.0)
        .inner_margin(14.0)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(RichText::new(label).color(MUTED).strong().size(12.0));
            ui.label(
                RichText::new(value.to_string())
                    .color(ACCENT)
                    .strong()
                    .size(28.0),
            );
            ui.label(RichText::new(suffix).color(MUTED).small());
        });
}

fn section_frame(ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(PANEL)
        .stroke(Stroke::new(1.0, PANEL_HOVER))
        .corner_radius(10.0)
        .inner_margin(16.0)
        .show(ui, content);
}

fn current_epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn session_completion_message(
    duration_minutes: u32,
    study_xp: u32,
    quest_xp: u32,
    daily_completion_bonus_xp: u32,
) -> String {
    let mut rewards = vec![format!("+{study_xp} study XP")];
    if quest_xp > 0 {
        rewards.push(format!("+{quest_xp} quest XP"));
    }
    if daily_completion_bonus_xp > 0 {
        rewards.push(format!("+{daily_completion_bonus_xp} daily clear XP"));
    }

    format!(
        "Session complete: {duration_minutes} min · {}",
        rewards.join(" · ")
    )
}

fn configure_style(context: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BACKGROUND;
    visuals.window_fill = BACKGROUND;
    visuals.selection.bg_fill = ACCENT;
    visuals.widgets.inactive.bg_fill = PANEL_HOVER;
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(42, 52, 78);
    visuals.widgets.active.bg_fill = ACCENT;
    context.set_visuals(visuals);
}

fn install_system_cjk_font(context: &egui::Context) {
    #[cfg(target_os = "macos")]
    let candidates = [
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/STHeiti Light.ttc",
    ];

    #[cfg(not(target_os = "macos"))]
    let candidates: [&str; 0] = [];

    let Some(font_bytes) = candidates.iter().find_map(|path| std::fs::read(path).ok()) else {
        return;
    };

    let mut fonts = FontDefinitions::default();
    let font_name = "system-cjk".to_string();
    fonts.font_data.insert(
        font_name.clone(),
        Arc::new(FontData::from_owned(font_bytes)),
    );
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .push(font_name.clone());
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .push(font_name);
    context.set_fonts(fonts);
}

#[cfg(test)]
mod tests {
    use super::session_completion_message;

    #[test]
    fn session_completion_feedback_keeps_reward_sources_separate() {
        let message = session_completion_message(30, 48, 100, 150);

        assert_eq!(
            message,
            "Session complete: 30 min · +48 study XP · +100 quest XP · +150 daily clear XP"
        );
    }

    #[test]
    fn session_completion_feedback_omits_rewards_that_were_not_earned() {
        let message = session_completion_message(25, 40, 0, 0);

        assert_eq!(message, "Session complete: 25 min · +40 study XP");
    }
}
