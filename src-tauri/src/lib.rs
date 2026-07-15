use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use study_rpg::{
    CalendarDate, CharacterClass, CompanionDisplay, CompanionMode, CompanionPreferences,
    CompanionWindowBounds, Dashboard, DesktopController, SqliteStore, StudySessionResult,
    StudyStatistics, StudyStatisticsReport, companion_window_bounds,
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
    title: String,
    energy: u8,
    level: u32,
    total_xp: u32,
    xp_into_level: u32,
    xp_for_next_level: u32,
    xp_progress_percent: u8,
    today_minutes: u32,
    total_sessions: u32,
    quests: Vec<QuestView>,
    daily_quest_completed: bool,
    daily_quest_reward_xp: u32,
    recent_sessions: Vec<SessionView>,
    active_session: Option<ActiveSessionView>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct QuestView {
    id: u64,
    title: String,
    current: u32,
    target: u32,
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
struct SessionResultView {
    topic: String,
    duration_minutes: u32,
    study_xp: u32,
    quest_reward_xp: u32,
    daily_completion_bonus_xp: u32,
    total_gained_xp: u32,
    completed_quests: Vec<String>,
    level_before: u32,
    level_after: u32,
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
fn get_statistics(state: State<'_, AppState>) -> Result<StatisticsView, String> {
    let controller = lock_controller(&state)?;
    Ok(StatisticsView::from(
        controller.statistics_at(current_epoch_seconds()),
    ))
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
fn start_session(topic: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut controller = lock_controller(&state)?;
    controller
        .start_session(&topic, current_epoch_seconds())
        .map_err(|error| error.to_string())
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
    Ok(DesktopController::load_or_create(
        store,
        "玩家",
        CharacterClass::Scholar,
        current_epoch_seconds(),
    )?)
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

impl From<Dashboard> for DashboardView {
    fn from(value: Dashboard) -> Self {
        Self {
            player_name: value.player_name,
            title: value.title,
            energy: value.energy,
            level: value.level.level,
            total_xp: value.total_xp,
            xp_into_level: value.level.xp_into_level,
            xp_for_next_level: value.level.xp_for_next_level,
            xp_progress_percent: value.xp_progress_percent,
            today_minutes: value.today_minutes,
            total_sessions: value.total_sessions,
            quests: value
                .quest_progress
                .into_iter()
                .map(|quest| QuestView {
                    id: quest.id,
                    title: quest.title,
                    current: quest.current,
                    target: quest.target,
                    reward_xp: quest.reward_xp,
                    completed: quest.completed,
                })
                .collect(),
            daily_quest_completed: value.daily_quest_completion.completed,
            daily_quest_reward_xp: value.daily_quest_completion.reward_xp,
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
            active_session: value.active_session.map(|session| ActiveSessionView {
                topic: session.topic,
                skill_name: session.skill_name,
                started_at_epoch_seconds: session.started_at_epoch_seconds,
                elapsed_minutes: session.elapsed_minutes,
                estimated_xp: session.estimated_xp,
            }),
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
                .map(|quest| quest.title)
                .collect(),
            level_before: value.player_xp.before.level,
            level_after: value.player_xp.after.level,
        }
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
            get_dashboard,
            get_statistics,
            get_companion_preferences,
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
