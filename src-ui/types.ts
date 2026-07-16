export type CompanionMode = "compact" | "expanded";
export type CharacterClassId = "scholar" | "engineer" | "mage" | "warrior" | "archer";

export interface StartupStateView {
  needsCharacterCreation: boolean;
  playerName?: string;
  playerClass?: CharacterClassId;
}

export interface QuestView {
  id: number;
  title: string;
  current: number;
  target: number;
  rewardXp: number;
  completed: boolean;
}

export interface SessionView {
  id: number;
  topic: string;
  skillName: string | null;
  durationMinutes: number;
  earnedXp: number;
}

export interface ActiveSessionView {
  topic: string;
  skillName: string | null;
  startedAtEpochSeconds: number;
  elapsedMinutes: number;
  estimatedXp: number;
}

export interface DashboardView {
  playerName: string;
  playerClass: CharacterClassId;
  title: string;
  energy: number;
  level: number;
  totalXp: number;
  xpIntoLevel: number;
  xpForNextLevel: number;
  xpProgressPercent: number;
  todayMinutes: number;
  totalSessions: number;
  quests: QuestView[];
  dailyQuestCompleted: boolean;
  dailyQuestRewardXp: number;
  recentSessions: SessionView[];
  activeSession: ActiveSessionView | null;
}

export interface CompanionPreferencesView {
  mode: CompanionMode;
  yPosition: number | null;
}

export interface SessionResultView {
  topic: string;
  durationMinutes: number;
  studyXp: number;
  questRewardXp: number;
  dailyCompletionBonusXp: number;
  totalGainedXp: number;
  completedQuests: string[];
  levelBefore: number;
  levelAfter: number;
}

export interface StatisticsPeriodView {
  totalSessions: number;
  totalMinutes: number;
  totalXp: number;
}

export interface DailyStatisticsView {
  epochDay: number;
  date: { year: number; month: number; day: number };
  statistics: StatisticsPeriodView;
}

export interface StatisticsView {
  today: StatisticsPeriodView;
  thisWeek: StatisticsPeriodView;
  thisMonth: StatisticsPeriodView;
  allTime: StatisticsPeriodView;
  lastSevenDays: DailyStatisticsView[];
  currentStreakDays: number;
  longestStreakDays: number;
}
