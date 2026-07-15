pub mod companion;
pub mod desktop;
pub mod player;
pub mod quest;
pub mod session;
pub mod skill;
pub mod statistics;
pub mod storage;
pub mod study_rpg;
pub mod xp;

pub use companion::{CompanionDisplay, CompanionWindowBounds, companion_window_bounds};
pub use desktop::{CompanionMode, CompanionPreferences, DesktopController, DesktopError};
pub use player::{CharacterClass, MAX_ENERGY, Player};
pub use session::{ActiveStudySession, StudySession};
pub use statistics::{CalendarDate, DailyStudyStatistics, StudyStatistics, StudyStatisticsReport};
pub use storage::SqliteStore;
pub use study_rpg::{
    Dashboard, DashboardActiveSession, DashboardDailyQuestCompletion, DashboardQuest,
    DashboardSession, StudyRpg, StudyRpgError, StudySessionInput, StudySessionResult,
    StudySessionStartInput,
};
