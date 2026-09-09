use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use study_rpg::{
    CalendarDate, CharacterClass, CompanionDisplay, CompanionMode, CompanionPreferences,
    CompanionWindowBounds, Dashboard, DashboardDailyQuestCompletion, DesktopController,
    GrowthEvent, GrowthEventKind, SqliteStore, StudySessionResult, StudyStatistics,
    StudyStatisticsReport, companion_window_bounds, quest::QuestTarget,
};
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{
    AppHandle, Manager, PhysicalPosition, PhysicalSize, RunEvent, State, WebviewWindow, WindowEvent,
};

struct AppState {
    controller: Mutex<DesktopController>,
    move_generation: AtomicU64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DashboardView {
    player_name: String,
    player_class: &'static str,
    title: String,
    energy: u8,
    level: u32,
    total_xp: u32,
    xp_into_level: u32,
    xp_for_next_level: u32,
    xp_progress_percent: u8,
    today_minutes: u32,
    total_sessions: u32,
    skills: Vec<SkillProgressView>,
    quests: Vec<QuestView>,
    daily_quest_status: DailyQuestStatusView,
    recent_sessions: Vec<SessionView>,
    growth_history: Vec<GrowthEventView>,
    active_session: Option<ActiveSessionView>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SkillProgressView {
    id: u64,
    name: String,
    parent_id: Option<u64>,
    depth: u32,
    unlocked: bool,
    level: u32,
    total_xp: u32,
    xp_into_level: u32,
    xp_for_next_level: u32,
    xp_progress_percent: u8,
    mastery_percent: u8,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GrowthEventView {
    id: u64,
    session_id: u64,
    topic: String,
    occurred_at_epoch_seconds: Option<u64>,
    details: GrowthEventDetailsView,
}

#[derive(Debug, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
enum GrowthEventDetailsView {
    PlayerLevelChange {
        gained_xp: u32,
        level_before: u32,
        level_after: u32,
        total_xp_after: u32,
    },
    SkillGrowth {
        skill_id: u64,
        skill_name: String,
        gained_xp: u32,
        level_before: u32,
        level_after: u32,
        total_xp_after: u32,
    },
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DailyQuestStatusView {
    completed: bool,
    completed_count: u32,
    total_count: u32,
    remaining_count: u32,
    progress_percent: u8,
    reward_xp: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct QuestView {
    id: u64,
    kind: &'static str,
    title: String,
    current: u32,
    target: u32,
    progress_percent: u8,
    reward_xp: u32,
    completed: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionView {
    id: u64,
    topic: String,
    skill_name: Option<String>,
    duration_minutes: u32,
    earned_xp: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ActiveSessionView {
    topic: String,
    skill_id: Option<u64>,
    skill_name: Option<String>,
    started_at_epoch_seconds: u64,
    elapsed_minutes: u32,
    estimated_xp: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CompanionPreferencesView {
    mode: &'static str,
    y_position: Option<i32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StartupStateView {
    needs_character_creation: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    player_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    player_class: Option<&'static str>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionResultView {
    topic: String,
    duration_minutes: u32,
    study_xp: u32,
    quest_reward_xp: u32,
    daily_completion_bonus_xp: u32,
    total_gained_xp: u32,
    completed_quests: Vec<CompletedQuestView>,
    level_before: u32,
    level_after: u32,
    growth_events: Vec<GrowthEventView>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CompletedQuestView {
    kind: &'static str,
    target: u32,
    title: String,
    reward_xp: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StatisticsView {
    today: StatisticsPeriodView,
    this_week: StatisticsPeriodView,
    this_month: StatisticsPeriodView,
    all_time: StatisticsPeriodView,
    last_seven_days: Vec<DailyStatisticsView>,
    current_streak_days: u32,
    longest_streak_days: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StatisticsPeriodView {
    total_sessions: u32,
    total_minutes: u32,
    total_xp: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DailyStatisticsView {
    epoch_day: u64,
    date: DateView,
    statistics: StatisticsPeriodView,
}

#[derive(Debug, Serialize)]
struct DateView {
    year: i64,
    month: u32,
    day: u32,
}

#[tauri::command]
fn window_kind(window: WebviewWindow) -> String {
    window.label().to_string()
}

#[tauri::command]
fn get_dashboard(state: State<'_, AppState>) -> Result<DashboardView, String> {
    let mut controller = lock_controller(&state)?;
    controller
        .dashboard_at(current_epoch_seconds())
        .map(DashboardView::from)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn get_startup_state(state: State<'_, AppState>) -> Result<StartupStateView, String> {
    let mut controller = lock_controller(&state)?;
    if controller.needs_character_creation() {
        return Ok(StartupStateView {
            needs_character_creation: true,
            player_name: None,
            player_class: None,
        });
    }
    let dashboard = controller
        .dashboard_at(current_epoch_seconds())
        .map_err(|error| error.to_string())?;
    Ok(StartupStateView {
        needs_character_creation: false,
        player_name: Some(dashboard.player_name),
        player_class: Some(character_class_id(dashboard.player_class)),
    })
}

#[tauri::command]
fn create_character(name: String, class: String, state: State<'_, AppState>) -> Result<(), String> {
    let class =
        character_class_from_id(&class).ok_or_else(|| "请选择有效的角色职业".to_string())?;
    let mut controller = lock_controller(&state)?;
    controller
        .create_character(name, class, current_epoch_seconds())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn get_statistics(state: State<'_, AppState>) -> Result<StatisticsView, String> {
    let controller = lock_controller(&state)?;
    controller
        .statistics_at(current_epoch_seconds())
        .map(StatisticsView::from)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn get_companion_preferences(
    state: State<'_, AppState>,
) -> Result<CompanionPreferencesView, String> {
    let controller = lock_controller(&state)?;
    Ok(CompanionPreferencesView::from(
        controller.companion_preferences(),
    ))
}

#[tauri::command]
fn create_skill(
    name: String,
    parent_id: Option<u64>,
    state: State<'_, AppState>,
) -> Result<u64, String> {
    lock_controller(&state)?
        .create_skill(&name, parent_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn start_session(
    topic: String,
    skill_name: Option<String>,
    skill_id: Option<u64>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut controller = lock_controller(&state)?;
    let skill_name = skill_name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty());
    match (skill_id, skill_name) {
        (Some(_), Some(_)) => Err("请选择已有技能或填写新技能名称".to_string()),
        (Some(id), None) => controller
            .start_session_with_skill_id(&topic, Some(id), current_epoch_seconds())
            .map_err(|error| error.to_string()),
        (None, name) => controller
            .start_session_with_skill(&topic, name, current_epoch_seconds())
            .map_err(|error| error.to_string()),
    }
}

#[tauri::command]
fn finish_session(state: State<'_, AppState>) -> Result<SessionResultView, String> {
    let mut controller = lock_controller(&state)?;
    controller
        .finish_session(current_epoch_seconds())
        .map(SessionResultView::from)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn set_companion_mode(
    mode: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<CompanionPreferencesView, String> {
    let mode = match mode.as_str() {
        "expanded" => CompanionMode::Expanded,
        "compact" => CompanionMode::Compact,
        _ => return Err("未知的 Companion 显示模式".to_string()),
    };
    let window = app
        .get_webview_window("companion")
        .ok_or_else(|| "Companion 窗口不存在".to_string())?;
    let current_y = window
        .outer_position()
        .map_err(|error| error.to_string())?
        .y;
    let requested = CompanionPreferences {
        mode,
        y_position: Some(current_y),
    };
    let bounds = bounds_for_window(&window, requested)?;
    let preferences = CompanionPreferences {
        mode,
        y_position: Some(bounds.y),
    };

    {
        let mut controller = lock_controller(&state)?;
        controller
            .set_companion_preferences(preferences)
            .map_err(|error| error.to_string())?;
    }
    apply_companion_bounds(&window, bounds)?;

    Ok(CompanionPreferencesView::from(preferences))
}

#[tauri::command]
fn open_dashboard(app: AppHandle) -> Result<(), String> {
    show_window(&app, "dashboard")
}

#[tauri::command]
fn open_companion(app: AppHandle) -> Result<(), String> {
    show_window(&app, "companion")
}

#[tauri::command]
fn hide_current_window(window: WebviewWindow) -> Result<(), String> {
    window.hide().map_err(|error| error.to_string())
}

#[tauri::command]
fn start_window_drag(window: WebviewWindow) -> Result<(), String> {
    window.start_dragging().map_err(|error| error.to_string())
}

fn lock_controller<'a>(
    state: &'a State<'_, AppState>,
) -> Result<MutexGuard<'a, DesktopController>, String> {
    state
        .controller
        .lock()
        .map_err(|_| "应用状态暂时不可用".to_string())
}

fn bounds_for_window(
    window: &WebviewWindow,
    preferences: CompanionPreferences,
) -> Result<CompanionWindowBounds, String> {
    let monitor = window
        .current_monitor()
        .map_err(|error| error.to_string())?
        .or_else(|| window.primary_monitor().ok().flatten())
        .ok_or_else(|| "无法确定 Companion 所在显示器".to_string())?;
    let work_area = monitor.work_area();
    Ok(companion_window_bounds(
        CompanionDisplay {
            x: work_area.position.x,
            y: work_area.position.y,
            width: work_area.size.width,
            height: work_area.size.height,
            scale_factor: monitor.scale_factor(),
        },
        preferences,
    ))
}

fn apply_companion_bounds(
    window: &WebviewWindow,
    bounds: CompanionWindowBounds,
) -> Result<(), String> {
    window
        .set_size(PhysicalSize::new(bounds.width, bounds.height))
        .map_err(|error| error.to_string())?;
    window
        .set_position(PhysicalPosition::new(bounds.x, bounds.y))
        .map_err(|error| error.to_string())
}

fn show_window(app: &AppHandle, label: &str) -> Result<(), String> {
    let window = app
        .get_webview_window(label)
        .ok_or_else(|| format!("窗口 {label} 不存在"))?;
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())
}

fn schedule_position_save(app: AppHandle, y_position: i32) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    let generation = state.move_generation.fetch_add(1, Ordering::Relaxed) + 1;

    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(250));
        let Some(state) = app.try_state::<AppState>() else {
            return;
        };
        if state.move_generation.load(Ordering::Relaxed) != generation {
            return;
        }
        if let Ok(mut controller) = state.controller.lock() {
            let mut preferences = controller.companion_preferences();
            preferences.y_position = Some(y_position);
            let _ = controller.set_companion_preferences(preferences);
        }
    });
}

fn install_window_behavior(app: &AppHandle) -> Result<(), String> {
    let companion = app
        .get_webview_window("companion")
        .ok_or_else(|| "Companion 窗口不存在".to_string())?;
    let preferences = app
        .state::<AppState>()
        .controller
        .lock()
        .map_err(|_| "应用状态暂时不可用".to_string())?
        .companion_preferences();
    let bounds = bounds_for_window(&companion, preferences)?;
    apply_companion_bounds(&companion, bounds)?;

    let companion_for_events = companion.clone();
    let handle_for_events = app.clone();
    companion.on_window_event(move |event| match event {
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            let _ = companion_for_events.hide();
        }
        WindowEvent::Moved(position) => {
            let mode = handle_for_events
                .try_state::<AppState>()
                .and_then(|state| {
                    state
                        .controller
                        .lock()
                        .ok()
                        .map(|controller| controller.companion_preferences().mode)
                })
                .unwrap_or(CompanionMode::Compact);
            let preferences = CompanionPreferences {
                mode,
                y_position: Some(position.y),
            };
            if let Ok(bounds) = bounds_for_window(&companion_for_events, preferences) {
                if position.x != bounds.x || position.y != bounds.y {
                    let _ = companion_for_events
                        .set_position(PhysicalPosition::new(bounds.x, bounds.y));
                }
                schedule_position_save(handle_for_events.clone(), bounds.y);
            }
        }
        _ => {}
    });

    if let Some(dashboard) = app.get_webview_window("dashboard") {
        let dashboard_for_events = dashboard.clone();
        dashboard.on_window_event(move |event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = dashboard_for_events.hide();
            }
        });
    }

    Ok(())
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let show_companion =
        MenuItem::with_id(app, "show_companion", "显示 Companion", true, None::<&str>)?;
    let show_dashboard =
        MenuItem::with_id(app, "show_dashboard", "打开 Dashboard", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "退出 Study RPG", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_companion, &show_dashboard, &separator, &quit])?;

    TrayIconBuilder::with_id("study-rpg")
        .icon(tray_icon())
        .icon_as_template(true)
        .tooltip("Study RPG")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show_companion" => {
                let _ = show_window(app, "companion");
            }
            "show_dashboard" => {
                let _ = show_window(app, "dashboard");
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    Ok(())
}

fn tray_icon() -> Image<'static> {
    const SIZE: u32 = 18;
    let mut rgba = vec![0_u8; (SIZE * SIZE * 4) as usize];
    for y in 2..16 {
        for x in 2..16 {
            let center = 8_i32;
            let distance = (x as i32 - center).abs() + (y as i32 - center).abs();
            let is_book = (4..=13).contains(&x) && (5..=13).contains(&y);
            let is_spark = distance <= 3;
            if is_book || is_spark {
                let offset = ((y * SIZE + x) * 4) as usize;
                rgba[offset..offset + 4].copy_from_slice(&[255, 255, 255, 255]);
            }
        }
    }
    Image::new_owned(rgba, SIZE, SIZE)
}

fn initialize_controller(app: &AppHandle) -> Result<DesktopController, Box<dyn std::error::Error>> {
    let app_data_dir = app.path().app_data_dir()?;
    fs::create_dir_all(&app_data_dir)?;
    let database_path = app_data_dir.join("study_rpg.sqlite3");
    migrate_legacy_database(&database_path)?;
    let store = SqliteStore::open(database_path)?;
    Ok(DesktopController::load(store, current_epoch_seconds())?)
}

fn migrate_legacy_database(destination: &PathBuf) -> std::io::Result<()> {
    let legacy = PathBuf::from("data/study_rpg.sqlite3");
    if !destination.exists() && legacy.exists() {
        fs::copy(legacy, destination)?;
    }
    Ok(())
}

fn current_epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn character_class_id(class: CharacterClass) -> &'static str {
    match class {
        CharacterClass::Scholar => "scholar",
        CharacterClass::Engineer => "engineer",
        CharacterClass::Mage => "mage",
        CharacterClass::Warrior => "warrior",
        CharacterClass::Archer => "archer",
    }
}

fn character_class_from_id(value: &str) -> Option<CharacterClass> {
    match value {
        "scholar" => Some(CharacterClass::Scholar),
        "engineer" => Some(CharacterClass::Engineer),
        "mage" => Some(CharacterClass::Mage),
        "warrior" => Some(CharacterClass::Warrior),
        "archer" => Some(CharacterClass::Archer),
        _ => None,
    }
}

impl From<Dashboard> for DashboardView {
    fn from(value: Dashboard) -> Self {
        Self {
            player_name: value.player_name,
            player_class: character_class_id(value.player_class),
            title: value.title,
            energy: value.energy,
            level: value.level.level,
            total_xp: value.total_xp,
            xp_into_level: value.level.xp_into_level,
            xp_for_next_level: value.level.xp_for_next_level,
            xp_progress_percent: value.xp_progress_percent,
            today_minutes: value.today_minutes,
            total_sessions: value.total_sessions,
            skills: value
                .skills
                .into_iter()
                .map(|skill| SkillProgressView {
                    id: skill.id,
                    name: skill.name,
                    parent_id: skill.parent_id,
                    depth: skill.depth,
                    unlocked: skill.unlocked,
                    level: skill.level,
                    total_xp: skill.total_xp,
                    xp_into_level: skill.xp_into_level,
                    xp_for_next_level: skill.xp_for_next_level,
                    xp_progress_percent: skill.xp_progress_percent,
                    mastery_percent: skill.mastery_percent,
                })
                .collect(),
            quests: value
                .quest_progress
                .into_iter()
                .map(|quest| {
                    let (kind, _) = quest_target_view(quest.quest_target);
                    QuestView {
                        id: quest.id,
                        kind,
                        title: quest.title,
                        current: quest.current,
                        target: quest.target,
                        progress_percent: quest.progress_percent,
                        reward_xp: quest.reward_xp,
                        completed: quest.completed,
                    }
                })
                .collect(),
            daily_quest_status: value.daily_quest_completion.into(),
            recent_sessions: value
                .recent_sessions
                .into_iter()
                .map(|session| SessionView {
                    id: session.id,
                    topic: session.topic,
                    skill_name: session.skill_name,
                    duration_minutes: session.duration_minutes,
                    earned_xp: session.earned_xp,
                })
                .collect(),
            growth_history: value
                .growth_history
                .into_iter()
                .map(GrowthEventView::from)
                .collect(),
            active_session: value.active_session.map(|session| ActiveSessionView {
                topic: session.topic,
                skill_id: session.skill_id,
                skill_name: session.skill_name,
                started_at_epoch_seconds: session.started_at_epoch_seconds,
                elapsed_minutes: session.elapsed_minutes,
                estimated_xp: session.estimated_xp,
            }),
        }
    }
}

impl From<GrowthEvent> for GrowthEventView {
    fn from(value: GrowthEvent) -> Self {
        let details = match value.kind {
            GrowthEventKind::PlayerLevelChange {
                gained_xp,
                level_before,
                level_after,
                total_xp_after,
            } => GrowthEventDetailsView::PlayerLevelChange {
                gained_xp,
                level_before,
                level_after,
                total_xp_after,
            },
            GrowthEventKind::SkillGrowth {
                skill_id,
                skill_name,
                gained_xp,
                level_before,
                level_after,
                total_xp_after,
            } => GrowthEventDetailsView::SkillGrowth {
                skill_id,
                skill_name,
                gained_xp,
                level_before,
                level_after,
                total_xp_after,
            },
        };

        Self {
            id: value.id,
            session_id: value.session_id,
            topic: value.topic,
            occurred_at_epoch_seconds: value.occurred_at_epoch_seconds,
            details,
        }
    }
}

impl From<CompanionPreferences> for CompanionPreferencesView {
    fn from(value: CompanionPreferences) -> Self {
        Self {
            mode: match value.mode {
                CompanionMode::Compact => "compact",
                CompanionMode::Expanded => "expanded",
            },
            y_position: value.y_position,
        }
    }
}

impl From<StudySessionResult> for SessionResultView {
    fn from(value: StudySessionResult) -> Self {
        Self {
            topic: value.session.topic,
            duration_minutes: value.session.duration_minutes,
            study_xp: value.session.earned_xp,
            quest_reward_xp: value.quest_reward_xp,
            daily_completion_bonus_xp: value.daily_completion_bonus_xp,
            total_gained_xp: value.player_xp.gained_xp,
            completed_quests: value
                .completed_quests
                .into_iter()
                .map(|quest| {
                    let (kind, target) = quest_target_view(quest.target);
                    CompletedQuestView {
                        kind,
                        target,
                        title: quest.title,
                        reward_xp: quest.reward_xp,
                    }
                })
                .collect(),
            level_before: value.player_xp.before.level,
            level_after: value.player_xp.after.level,
            growth_events: value
                .growth_events
                .into_iter()
                .map(GrowthEventView::from)
                .collect(),
        }
    }
}

impl From<DashboardDailyQuestCompletion> for DailyQuestStatusView {
    fn from(value: DashboardDailyQuestCompletion) -> Self {
        Self {
            completed: value.completed,
            completed_count: value.completed_count,
            total_count: value.total_count,
            remaining_count: value.remaining_count,
            progress_percent: value.progress_percent,
            reward_xp: value.reward_xp,
        }
    }
}

fn quest_target_view(target: QuestTarget) -> (&'static str, u32) {
    match target {
        QuestTarget::StudyMinutes(target) => ("studyMinutes", target),
        QuestTarget::CompleteSessions(target) => ("completeSessions", target),
    }
}

impl From<StudyStatisticsReport> for StatisticsView {
    fn from(value: StudyStatisticsReport) -> Self {
        Self {
            today: value.today.into(),
            this_week: value.this_week.into(),
            this_month: value.this_month.into(),
            all_time: value.all_time.into(),
            last_seven_days: value
                .last_seven_days
                .into_iter()
                .map(|day| DailyStatisticsView {
                    epoch_day: day.epoch_day,
                    date: day.date.into(),
                    statistics: day.statistics.into(),
                })
                .collect(),
            current_streak_days: value.current_streak_days,
            longest_streak_days: value.longest_streak_days,
        }
    }
}

impl From<StudyStatistics> for StatisticsPeriodView {
    fn from(value: StudyStatistics) -> Self {
        Self {
            total_sessions: value.total_sessions,
            total_minutes: value.total_minutes,
            total_xp: value.total_xp,
        }
    }
}

impl From<CalendarDate> for DateView {
    fn from(value: CalendarDate) -> Self {
        Self {
            year: value.year,
            month: value.month,
            day: value.day,
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let controller = initialize_controller(app.handle())?;
            app.manage(AppState {
                controller: Mutex::new(controller),
                move_generation: AtomicU64::new(0),
            });
            install_window_behavior(app.handle())?;
            build_tray(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            window_kind,
            get_startup_state,
            create_character,
            get_dashboard,
            get_statistics,
            get_companion_preferences,
            create_skill,
            start_session,
            finish_session,
            set_companion_mode,
            open_dashboard,
            open_companion,
            hide_current_window,
            start_window_drag,
        ])
        .build(tauri::generate_context!())
        .expect("无法构建 Study RPG 桌面应用");

    app.run(|_, event| {
        if let RunEvent::ExitRequested { api, code, .. } = event
            && code.is_none()
        {
            api.prevent_exit();
        }
    });
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};
    use study_rpg::{CharacterClass, DesktopController, SqliteStore, StudyRpg, StudySessionInput};
    use tauri::WebviewWindowBuilder;
    use tauri::webview::InvokeRequest;

    use super::{
        AppState, DashboardView, SessionResultView, create_character, create_skill, get_dashboard,
        get_startup_state, start_session,
    };

    #[test]
    fn session_result_view_keeps_each_completed_quest_reward_for_visual_feedback() {
        let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);
        let result = app.complete_study_session(StudySessionInput {
            topic: "Complete today's quests".to_string(),
            skill_id: None,
            duration_minutes: 30,
        });

        let view = serde_json::to_value(SessionResultView::from(result)).unwrap();

        assert_eq!(
            view["completedQuests"],
            json!([
                {
                    "kind": "studyMinutes",
                    "target": 30,
                    "title": "Study 30 minutes",
                    "rewardXp": 60
                },
                {
                    "kind": "completeSessions",
                    "target": 1,
                    "title": "Complete 1 study session",
                    "rewardXp": 40
                }
            ])
        );
    }

    #[test]
    fn session_result_view_exposes_this_session_growth_events() {
        let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);
        let rust = app.add_skill("Rust", None);
        let result = app.complete_study_session(StudySessionInput {
            topic: "Rust ownership".to_string(),
            skill_id: Some(rust),
            duration_minutes: 30,
        });

        let view = serde_json::to_value(SessionResultView::from(result)).unwrap();

        assert_eq!(view["growthEvents"][0]["details"]["kind"], "skillGrowth");
        assert_eq!(view["growthEvents"][0]["details"]["skillName"], "Rust");
        assert_eq!(view["growthEvents"][0]["details"]["gainedXp"], 48);
        assert_eq!(
            view["growthEvents"][1]["details"]["kind"],
            "playerLevelChange"
        );
        assert_eq!(view["growthEvents"][1]["details"]["levelBefore"], 1);
        assert_eq!(view["growthEvents"][1]["details"]["levelAfter"], 3);
    }

    #[test]
    fn dashboard_view_keeps_core_daily_quest_progress_for_ui_rendering() {
        let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);
        app.complete_study_session(StudySessionInput {
            topic: "Partial daily progress".to_string(),
            skill_id: None,
            duration_minutes: 15,
        });

        let view = serde_json::to_value(DashboardView::from(app.dashboard())).unwrap();

        assert_eq!(view["dailyQuestStatus"]["completedCount"], 1);
        assert_eq!(view["dailyQuestStatus"]["totalCount"], 2);
        assert_eq!(view["dailyQuestStatus"]["remainingCount"], 1);
        assert_eq!(view["dailyQuestStatus"]["progressPercent"], 75);
        assert_eq!(view["quests"][0]["kind"], "studyMinutes");
        assert_eq!(view["quests"][0]["progressPercent"], 50);
        assert_eq!(view["quests"][1]["progressPercent"], 100);
    }

    #[test]
    fn dashboard_view_exposes_skill_progress_and_growth_history() {
        let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);
        let rust = app.add_skill("Rust", None);
        app.complete_study_session(StudySessionInput {
            topic: "Rust ownership".to_string(),
            skill_id: Some(rust),
            duration_minutes: 30,
        });

        let view = serde_json::to_value(DashboardView::from(app.dashboard())).unwrap();

        assert_eq!(view["skills"][0]["name"], "Rust");
        assert_eq!(view["skills"][0]["totalXp"], 48);
        assert_eq!(
            view["growthHistory"][0]["details"]["kind"],
            "playerLevelChange"
        );
        assert_eq!(view["growthHistory"][1]["details"]["kind"], "skillGrowth");
    }

    #[test]
    fn ipc_creates_the_first_run_character_and_reports_the_ready_identity() {
        let controller = DesktopController::load(SqliteStore::in_memory().unwrap(), 1_000).unwrap();
        let app = tauri::test::mock_builder()
            .manage(AppState {
                controller: std::sync::Mutex::new(controller),
                move_generation: std::sync::atomic::AtomicU64::new(0),
            })
            .invoke_handler(tauri::generate_handler![
                get_startup_state,
                create_character
            ])
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();
        let webview = WebviewWindowBuilder::new(&app, "companion", Default::default())
            .build()
            .unwrap();

        let before = invoke(&webview, "get_startup_state", json!({}));
        assert_eq!(before, json!({ "needsCharacterCreation": true }));

        invoke(
            &webview,
            "create_character",
            json!({ "name": "Nembx", "class": "engineer" }),
        );
        let after = invoke(&webview, "get_startup_state", json!({}));

        assert_eq!(
            after,
            json!({
                "needsCharacterCreation": false,
                "playerName": "Nembx",
                "playerClass": "engineer"
            })
        );
    }

    #[test]
    fn ipc_starts_a_session_with_an_optional_skill_name() {
        let controller = DesktopController::load_or_create(
            SqliteStore::in_memory().unwrap(),
            "Nembx",
            CharacterClass::Scholar,
            1_000,
        )
        .unwrap();
        let app = tauri::test::mock_builder()
            .manage(AppState {
                controller: std::sync::Mutex::new(controller),
                move_generation: std::sync::atomic::AtomicU64::new(0),
            })
            .invoke_handler(tauri::generate_handler![start_session, get_dashboard])
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();
        let webview = WebviewWindowBuilder::new(&app, "companion", Default::default())
            .build()
            .unwrap();

        invoke(
            &webview,
            "start_session",
            json!({ "topic": "Rust ownership", "skillName": "Rust" }),
        );
        let dashboard = invoke(&webview, "get_dashboard", json!({}));

        assert_eq!(dashboard["activeSession"]["skillName"], "Rust");
        assert_eq!(dashboard["skills"][0]["name"], "Rust");
    }

    #[test]
    fn ipc_creates_a_skill_branch_and_starts_learning_the_selected_child() {
        let controller = DesktopController::load_or_create(
            SqliteStore::in_memory().unwrap(),
            "Nembx",
            CharacterClass::Scholar,
            1_000,
        )
        .unwrap();
        let app = tauri::test::mock_builder()
            .manage(AppState {
                controller: std::sync::Mutex::new(controller),
                move_generation: std::sync::atomic::AtomicU64::new(0),
            })
            .invoke_handler(tauri::generate_handler![
                create_skill,
                start_session,
                get_dashboard
            ])
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();
        let webview = WebviewWindowBuilder::new(&app, "companion", Default::default())
            .build()
            .unwrap();

        let parent = invoke(
            &webview,
            "create_skill",
            json!({ "name": "编程", "parentId": null }),
        );
        let child = invoke(
            &webview,
            "create_skill",
            json!({ "name": "Rust", "parentId": parent }),
        );
        invoke(
            &webview,
            "create_skill",
            json!({ "name": "Rust", "parentId": null }),
        );
        invoke(
            &webview,
            "start_session",
            json!({ "topic": "Rust ownership", "skillId": child }),
        );
        let dashboard = invoke(&webview, "get_dashboard", json!({}));

        assert_eq!(dashboard["activeSession"]["skillId"], child);
        assert_eq!(dashboard["activeSession"]["skillName"], "Rust");
        assert_eq!(dashboard["skills"][0]["parentId"], Value::Null);
        assert_eq!(dashboard["skills"][0]["depth"], 0);
        assert_eq!(dashboard["skills"][1]["id"], child);
        assert_eq!(dashboard["skills"][1]["parentId"], parent);
        assert_eq!(dashboard["skills"][1]["depth"], 1);
        assert_eq!(dashboard["skills"][1]["unlocked"], false);
        assert_eq!(dashboard["skills"].as_array().unwrap().len(), 3);
    }

    fn invoke(
        webview: &tauri::WebviewWindow<tauri::test::MockRuntime>,
        command: &str,
        body: Value,
    ) -> Value {
        tauri::test::get_ipc_response(
            webview,
            InvokeRequest {
                cmd: command.into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "tauri://localhost".parse().unwrap(),
                body: tauri::ipc::InvokeBody::Json(body),
                headers: Default::default(),
                invoke_key: tauri::test::INVOKE_KEY.to_string(),
            },
        )
        .unwrap()
        .deserialize::<Value>()
        .unwrap()
    }
}
