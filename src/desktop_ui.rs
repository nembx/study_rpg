use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use eframe::egui::{self, Color32, FontData, FontDefinitions, FontFamily, RichText, Stroke};
use study_rpg::{Dashboard, DesktopController, MAX_ENERGY, StudyStatistics, StudyStatisticsReport};

const BACKGROUND: Color32 = Color32::from_rgb(12, 16, 28);
const SIDEBAR: Color32 = Color32::from_rgb(15, 20, 34);
const PANEL: Color32 = Color32::from_rgb(24, 30, 48);
const PANEL_HOVER: Color32 = Color32::from_rgb(31, 39, 61);
const ACCENT: Color32 = Color32::from_rgb(250, 190, 72);
const BLUE: Color32 = Color32::from_rgb(102, 166, 255);
const PURPLE: Color32 = Color32::from_rgb(174, 132, 255);
const SUCCESS: Color32 = Color32::from_rgb(91, 214, 145);
const DANGER: Color32 = Color32::from_rgb(244, 112, 128);
const MUTED: Color32 = Color32::from_rgb(155, 166, 190);

pub struct StudyRpgDesktopApp {
    controller: DesktopController,
    page: DesktopPage,
    topic: String,
    feedback: Option<Feedback>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DesktopPage {
    Dashboard,
    Statistics,
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
            page: DesktopPage::Dashboard,
            topic: String::new(),
            feedback: None,
        }
    }

    fn render_header(ui: &mut egui::Ui, dashboard: &Dashboard) {
        egui::Frame::new()
            .fill(Color32::from_rgb(28, 35, 57))
            .stroke(Stroke::new(1.0, ACCENT.gamma_multiply(0.35)))
            .corner_radius(16.0)
            .inner_margin(20.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new("今日冒险").color(ACCENT).strong().size(13.0));
                        ui.label(
                            RichText::new(format!(
                                "欢迎回来，{}",
                                localized_player_name(&dashboard.player_name)
                            ))
                            .strong()
                            .size(30.0),
                        );
                        ui.label(
                            RichText::new(localized_title(&dashboard.title))
                                .color(MUTED)
                                .size(15.0),
                        );
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        egui::Frame::new()
                            .fill(ACCENT.gamma_multiply(0.13))
                            .stroke(Stroke::new(1.0, ACCENT.gamma_multiply(0.55)))
                            .corner_radius(14.0)
                            .inner_margin(14.0)
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new(format!("等级 {}", dashboard.level.level))
                                        .color(ACCENT)
                                        .strong()
                                        .size(20.0),
                                );
                            });
                    });
                });

                ui.add_space(14.0);
                ui.add(
                    egui::ProgressBar::new(f32::from(dashboard.xp_progress_percent) / 100.0)
                        .desired_width(ui.available_width())
                        .fill(ACCENT)
                        .text(format!(
                            "距离下一级：{} / {} XP",
                            dashboard.level.xp_into_level, dashboard.level.xp_for_next_level
                        )),
                );
            });
    }

    fn render_stat_cards(ui: &mut egui::Ui, dashboard: &Dashboard) {
        ui.columns(3, |columns| {
            stat_card(&mut columns[0], "今日学习", dashboard.today_minutes, "分钟");
            stat_card(&mut columns[1], "完成学习", dashboard.total_sessions, "次");
            stat_card(&mut columns[2], "累计经验", dashboard.total_xp, "XP");
        });
    }

    fn render_sidebar(
        &mut self,
        ui: &mut egui::Ui,
        dashboard: &Dashboard,
        current_streak_days: u32,
    ) {
        egui::Frame::new()
            .fill(SIDEBAR)
            .stroke(Stroke::new(1.0, PANEL_HOVER))
            .inner_margin(18.0)
            .show(ui, |ui| {
                ui.set_min_width(180.0);
                ui.set_min_height(ui.available_height());
                ui.label(RichText::new("✦").color(ACCENT).size(30.0));
                ui.label(RichText::new("学习冒险").strong().size(22.0));
                ui.label(RichText::new("把每一次专注变成成长").color(MUTED).small());
                ui.add_space(24.0);

                navigation_button(ui, &mut self.page, DesktopPage::Dashboard, "◈  冒险主页");
                ui.add_space(8.0);
                navigation_button(ui, &mut self.page, DesktopPage::Statistics, "▥  学习统计");

                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    egui::Frame::new()
                        .fill(PANEL)
                        .corner_radius(12.0)
                        .inner_margin(12.0)
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.label(
                                RichText::new(localized_player_name(&dashboard.player_name))
                                    .strong(),
                            );
                            ui.label(
                                RichText::new(format!(
                                    "等级 {} · {}",
                                    dashboard.level.level,
                                    localized_title(&dashboard.title)
                                ))
                                .color(MUTED)
                                .small(),
                            );
                            ui.label(
                                RichText::new(format!("连续学习 {} 天", current_streak_days))
                                    .color(SUCCESS)
                                    .small(),
                            );
                            ui.add(
                                egui::ProgressBar::new(
                                    (f32::from(dashboard.energy) / f32::from(MAX_ENERGY))
                                        .clamp(0.0, 1.0),
                                )
                                .desired_width(ui.available_width())
                                .fill(BLUE)
                                .text(format!("活力 {}", dashboard.energy)),
                            );
                        });
                });
            });
    }

    fn render_session_panel(&mut self, ui: &mut egui::Ui, dashboard: &Dashboard, now: u64) {
        section_frame(ui, |ui| {
            ui.label(RichText::new("专注学习").color(MUTED).strong());
            ui.add_space(8.0);

            if let Some(active) = &dashboard.active_session {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new(&active.topic).strong().size(22.0));
                        ui.label(
                            RichText::new(format!(
                                "已专注 {} · 预计获得 {} XP",
                                elapsed_timer_text(active.started_at_epoch_seconds, now),
                                active.estimated_xp
                            ))
                            .color(MUTED),
                        );
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(egui::Button::new(RichText::new("完成学习").strong()).fill(DANGER))
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
                        egui::TextEdit::singleline(&mut self.topic).hint_text("今天准备学习什么？"),
                    );
                    let enter_pressed =
                        input.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
                    let start_clicked = ui
                        .add_sized(
                            [118.0, 38.0],
                            egui::Button::new(RichText::new("开始学习").color(BACKGROUND).strong())
                                .fill(ACCENT),
                        )
                        .clicked();

                    if enter_pressed || start_clicked {
                        match self.controller.start_session(&self.topic, now) {
                            Ok(()) => {
                                let topic = self.topic.trim().to_string();
                                self.topic.clear();
                                self.feedback = Some(Feedback {
                                    message: format!("已开始学习：{topic}"),
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
                ui.label(RichText::new("每日任务").color(MUTED).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!(
                            "全清奖励 {} XP",
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
                            ui.label(RichText::new(localized_quest_title(&quest.title)).strong());
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
                        "今日任务已全清 · 已领取 {} XP",
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
            ui.label(RichText::new("最近学习").color(MUTED).strong());
            ui.add_space(8.0);

            if dashboard.recent_sessions.is_empty() {
                ui.label(RichText::new("完成第一次学习，开启你的成长旅程。").color(MUTED));
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
                            RichText::new(format!("{} 分钟", session.duration_minutes))
                                .color(MUTED),
                        );
                    });
                });
                ui.separator();
            }
        });
    }

    fn render_statistics(ui: &mut egui::Ui, report: &StudyStatisticsReport) {
        ui.columns(4, |columns| {
            statistics_period_card(&mut columns[0], "今日", &report.today);
            statistics_period_card(&mut columns[1], "本周", &report.this_week);
            statistics_period_card(&mut columns[2], "本月", &report.this_month);
            statistics_period_card(&mut columns[3], "累计", &report.all_time);
        });

        ui.add_space(12.0);
        ui.columns(2, |columns| {
            stat_card(
                &mut columns[0],
                "当前连续学习",
                report.current_streak_days,
                "天",
            );
            stat_card(
                &mut columns[1],
                "最长连续学习",
                report.longest_streak_days,
                "天",
            );
        });

        ui.add_space(12.0);
        render_activity_chart(ui, report);
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
                        ui.heading("学习冒险无法读取本地数据");
                        ui.label(RichText::new(error.to_string()).color(DANGER));
                    });
                return;
            }
        };
        let statistics = self.controller.statistics_at(now);
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(BACKGROUND).inner_margin(0.0))
            .show(ui, |ui| {
                let available_height = ui.available_height();
                ui.horizontal_top(|ui| {
                    ui.allocate_ui_with_layout(
                        egui::vec2(216.0, available_height),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| self.render_sidebar(ui, &dashboard, statistics.current_streak_days),
                    );

                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), available_height),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            egui::Frame::new().inner_margin(24.0).show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                egui::ScrollArea::vertical().show(ui, |ui| {
                                    Self::render_header(ui, &dashboard);
                                    ui.add_space(18.0);
                                    match self.page {
                                        DesktopPage::Dashboard => {
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
                                                Self::render_recent_sessions(
                                                    &mut columns[1],
                                                    &dashboard,
                                                );
                                            });
                                        }
                                        DesktopPage::Statistics => {
                                            Self::render_statistics(ui, &statistics);
                                        }
                                    }
                                });
                            });
                        },
                    );
                });
            });
    }
}

fn navigation_button(ui: &mut egui::Ui, page: &mut DesktopPage, target: DesktopPage, label: &str) {
    let selected = *page == target;
    let text_color = if selected { ACCENT } else { MUTED };
    let fill = if selected {
        ACCENT.gamma_multiply(0.12)
    } else {
        Color32::TRANSPARENT
    };
    let stroke = if selected {
        Stroke::new(1.0, ACCENT.gamma_multiply(0.45))
    } else {
        Stroke::NONE
    };

    if ui
        .add_sized(
            [ui.available_width(), 42.0],
            egui::Button::new(RichText::new(label).color(text_color).strong())
                .fill(fill)
                .stroke(stroke)
                .corner_radius(10.0),
        )
        .clicked()
    {
        *page = target;
    }
}

fn card_accent(label: &str) -> Color32 {
    if label.contains("经验") || label.contains("累计") {
        ACCENT
    } else if label.contains("连续") {
        SUCCESS
    } else if label.contains("周") || label.contains("完成") {
        BLUE
    } else if label.contains("月") {
        PURPLE
    } else {
        ACCENT
    }
}

fn statistics_period_card(ui: &mut egui::Ui, label: &str, statistics: &StudyStatistics) {
    let color = card_accent(label);
    egui::Frame::new()
        .fill(PANEL)
        .stroke(Stroke::new(1.0, color.gamma_multiply(0.35)))
        .corner_radius(14.0)
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(RichText::new(label).color(MUTED).strong().size(12.0));
            ui.label(
                RichText::new(format!("{} 分钟", statistics.total_minutes))
                    .color(color)
                    .strong()
                    .size(24.0),
            );
            ui.label(
                RichText::new(format!(
                    "{} 次学习 · {} XP",
                    statistics.total_sessions, statistics.total_xp
                ))
                .color(MUTED)
                .small(),
            );
        });
}

fn render_activity_chart(ui: &mut egui::Ui, report: &StudyStatisticsReport) {
    section_frame(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("最近七天").color(MUTED).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new("学习时长柱 · XP 趋势线").color(MUTED).small());
            });
        });
        ui.add_space(10.0);

        let desired_size = egui::vec2(ui.available_width(), 220.0);
        let (rect, _) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
        let painter = ui.painter_at(rect);
        let chart_top = rect.top() + 24.0;
        let chart_bottom = rect.bottom() - 28.0;
        let maximum_minutes = report
            .last_seven_days
            .iter()
            .map(|day| day.statistics.total_minutes)
            .max()
            .unwrap_or(0);
        let maximum_xp = report
            .last_seven_days
            .iter()
            .map(|day| day.statistics.total_xp)
            .max()
            .unwrap_or(0);
        let slot_width = rect.width() / report.last_seven_days.len().max(1) as f32;
        let mut xp_points = Vec::with_capacity(report.last_seven_days.len());

        for (index, day) in report.last_seven_days.iter().enumerate() {
            let center_x = rect.left() + slot_width * (index as f32 + 0.5);
            let fraction = activity_bar_fraction(day.statistics.total_minutes, maximum_minutes);
            let bar_height = (chart_bottom - chart_top) * fraction;
            let bar_rect = egui::Rect::from_min_max(
                egui::pos2(center_x - slot_width * 0.28, chart_bottom - bar_height),
                egui::pos2(center_x + slot_width * 0.28, chart_bottom),
            );
            painter.rect_filled(bar_rect, 5.0, ACCENT);
            painter.text(
                egui::pos2(center_x, chart_bottom - bar_height - 5.0),
                egui::Align2::CENTER_BOTTOM,
                format!("{}分", day.statistics.total_minutes),
                egui::FontId::proportional(12.0),
                MUTED,
            );
            let xp_fraction = activity_bar_fraction(day.statistics.total_xp, maximum_xp);
            xp_points.push(egui::pos2(
                center_x,
                chart_bottom - (chart_bottom - chart_top) * xp_fraction,
            ));
            painter.text(
                egui::pos2(center_x, rect.bottom() - 4.0),
                egui::Align2::CENTER_BOTTOM,
                format!("{:02}-{:02}", day.date.month, day.date.day),
                egui::FontId::proportional(12.0),
                MUTED,
            );
        }

        if maximum_xp > 0 {
            for points in xp_points.windows(2) {
                painter.line_segment([points[0], points[1]], Stroke::new(2.0, SUCCESS));
            }
            for point in xp_points {
                painter.circle_filled(point, 4.0, SUCCESS);
            }
        }
    });
}

fn stat_card(ui: &mut egui::Ui, label: &str, value: u32, suffix: &str) {
    let color = card_accent(label);
    egui::Frame::new()
        .fill(PANEL)
        .stroke(Stroke::new(1.0, color.gamma_multiply(0.35)))
        .corner_radius(14.0)
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(RichText::new(label).color(MUTED).strong().size(12.0));
            ui.label(
                RichText::new(value.to_string())
                    .color(color)
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
        .corner_radius(14.0)
        .inner_margin(18.0)
        .show(ui, content);
}

fn current_epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn elapsed_timer_text(started_at_epoch_seconds: u64, current_epoch_seconds: u64) -> String {
    let elapsed_seconds = current_epoch_seconds.saturating_sub(started_at_epoch_seconds);
    let hours = elapsed_seconds / 3_600;
    let minutes = (elapsed_seconds % 3_600) / 60;
    let seconds = elapsed_seconds % 60;

    if hours == 0 {
        format!("{minutes:02}:{seconds:02}")
    } else {
        format!("{hours}:{minutes:02}:{seconds:02}")
    }
}

fn activity_bar_fraction(minutes: u32, maximum_minutes: u32) -> f32 {
    if maximum_minutes == 0 {
        return 0.0;
    }

    (minutes as f32 / maximum_minutes as f32).clamp(0.0, 1.0)
}

fn session_completion_message(
    duration_minutes: u32,
    study_xp: u32,
    quest_xp: u32,
    daily_completion_bonus_xp: u32,
) -> String {
    let mut rewards = vec![format!("学习 +{study_xp} XP")];
    if quest_xp > 0 {
        rewards.push(format!("任务 +{quest_xp} XP"));
    }
    if daily_completion_bonus_xp > 0 {
        rewards.push(format!("全清 +{daily_completion_bonus_xp} XP"));
    }

    format!(
        "学习完成：{duration_minutes} 分钟 · {}",
        rewards.join(" · ")
    )
}

fn localized_player_name(name: &str) -> &str {
    if name == "Player" { "玩家" } else { name }
}

fn localized_title(title: &str) -> &str {
    match title {
        "Novice Learner" => "见习学者",
        "Knowledge Hunter" => "知识猎手",
        "Scholar Adventurer" => "学识冒险家",
        "Master Student" => "求学大师",
        "Legendary Learner" => "传奇求知者",
        _ => title,
    }
}

fn localized_quest_title(title: &str) -> String {
    if let Some(minutes) = title
        .strip_prefix("Study ")
        .and_then(|value| value.strip_suffix(" minutes"))
    {
        return format!("学习 {minutes} 分钟");
    }
    if let Some(sessions) = title
        .strip_prefix("Complete ")
        .and_then(|value| value.strip_suffix(" study session"))
    {
        return format!("完成 {sessions} 次学习");
    }

    title.to_string()
}

fn configure_style(context: &egui::Context) {
    let mut style = (*context.style_of(egui::Theme::Dark)).clone();
    style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    style.spacing.button_padding = egui::vec2(14.0, 9.0);
    context.set_style_of(egui::Theme::Dark, style);

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
    use super::{elapsed_timer_text, session_completion_message};

    #[test]
    fn active_timer_feedback_includes_running_seconds() {
        assert_eq!(elapsed_timer_text(1_000, 1_000 + 25 * 60 + 3), "25:03");
        assert_eq!(elapsed_timer_text(1_000, 1_000 + 60 * 60), "1:00:00");
    }

    #[test]
    fn session_completion_feedback_keeps_reward_sources_separate() {
        let message = session_completion_message(30, 48, 100, 150);

        assert_eq!(
            message,
            "学习完成：30 分钟 · 学习 +48 XP · 任务 +100 XP · 全清 +150 XP"
        );
    }

    #[test]
    fn session_completion_feedback_omits_rewards_that_were_not_earned() {
        let message = session_completion_message(25, 40, 0, 0);

        assert_eq!(message, "学习完成：25 分钟 · 学习 +40 XP");
    }
}
