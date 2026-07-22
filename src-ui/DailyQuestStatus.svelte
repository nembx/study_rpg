<script lang="ts">
  import type { DailyQuestStatusView } from "./types";

  export let status: DailyQuestStatusView;
  export let completedMessage: string;
</script>

<div class="quest-progress-summary">
  <div>
    <span>{status.completed ? "今日进度已完成" : `还差 ${status.remainingCount} 项任务`}</span>
    <strong>{status.progressPercent}%</strong>
  </div>
  <div
    class="progress-track"
    role="progressbar"
    aria-label={`今日任务进度 ${status.completedCount}/${status.totalCount}`}
    aria-valuemin="0"
    aria-valuemax="100"
    aria-valuenow={status.progressPercent}
  >
    <div style={`width: ${status.progressPercent}%`}></div>
  </div>
</div>

<slot />

<div class:complete={status.completed} class="daily-bonus-card">
  <div class="daily-bonus-icon">{status.completed ? "✦" : "◇"}</div>
  <div>
    <span>{status.completed ? "DAILY COMPLETE" : "ALL CLEAR BONUS"}</span>
    <strong>{status.completed ? "今日任务全清" : "完成全部任务"}</strong>
    <small>{status.completed ? completedMessage : `还差 ${status.remainingCount} 项即可领取`}</small>
  </div>
  <em>+{status.rewardXp} XP</em>
</div>
