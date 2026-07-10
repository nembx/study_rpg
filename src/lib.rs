pub mod desktop;
pub mod player;
pub mod quest;
pub mod session;
pub mod skill;
pub mod statistics;
pub mod storage;
pub mod study_rpg;
pub mod xp;

pub use desktop::{DesktopController, DesktopError};
pub use player::{CharacterClass, Player};
pub use session::{ActiveStudySession, StudySession};
pub use statistics::{DailyStudyStatistics, StudyStatistics, StudyStatisticsReport};
pub use storage::SqliteStore;
pub use study_rpg::{
    Dashboard, DashboardActiveSession, DashboardDailyQuestCompletion, DashboardQuest,
    DashboardSession, StudyRpg, StudyRpgError, StudySessionInput, StudySessionResult,
    StudySessionStartInput,
};
