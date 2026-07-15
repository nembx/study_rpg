<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import type {
    CompanionMode,
    CompanionPreferencesView,
    DashboardView,
    SessionResultView,
    StatisticsPeriodView,
    StatisticsView,
  } from "./types";

  let windowKind = "";
  let dashboard: DashboardView | null = null;
  let statistics: StatisticsView | null = null;
  let mode: CompanionMode = "compact";
  let topic = "";
  let errorMessage = "";
  let feedback: SessionResultView | null = null;
  let busy = false;
  let nowSeconds = Math.floor(Date.now() / 1000);
  let refreshCounter = 0;
  let feedbackTimer: ReturnType<typeof setTimeout> | undefined;

  $: recentTopics = dashboard
    ? [...new Set(dashboard.recentSessions.map((session) => session.topic))].slice(0, 3)
    : [];
  $: activeSeconds = dashboard?.activeSession
    ? Math.max(0, nowSeconds - dashboard.activeSession.startedAtEpochSeconds)
    : 0;

  onMount(() => {
    void initialize();
    const timer = setInterval(() => {
      nowSeconds = Math.floor(Date.now() / 1000);
      refreshCounter += 1;
      if (refreshCounter % 5 === 0) void refreshData(false);
    }, 1000);

    return () => {
      clearInterval(timer);
      if (feedbackTimer) clearTimeout(feedbackTimer);
    };
  });

  async function initialize() {
    try {
      windowKind = await invoke<string>("window_kind");
      if (windowKind === "companion") {
        const preferences = await invoke<CompanionPreferencesView>("get_companion_preferences");
        mode = preferences.mode;
      }
      await refreshData(true);
    } catch (error) {
      setError(error);
    }
  }

  async function refreshData(includeStatistics: boolean) {
    try {
      dashboard = await invoke<DashboardView>("get_dashboard");
      if (includeStatistics || windowKind === "dashboard") {
        statistics = await invoke<StatisticsView>("get_statistics");
      }
      errorMessage = "";
    } catch (error) {
      setError(error);
    }
  }

  async function setMode(nextMode: CompanionMode) {
    try {
      const preferences = await invoke<CompanionPreferencesView>("set_companion_mode", {
        mode: nextMode,
      });
      mode = preferences.mode;
    } catch (error) {
      setError(error);
    }
  }

  async function beginSession(chosenTopic = topic) {
    const trimmed = chosenTopic.trim();
    if (!trimmed) {
      await setMode("expanded");
      errorMessage = "请先输入学习主题";
      return;
    }

    busy = true;
    try {
      await invoke("start_session", { topic: trimmed });
      topic = "";
      feedback = null;
      errorMessage = "";
      await refreshData(false);
    } catch (error) {
      setError(error);
    } finally {
      busy = false;
    }
  }

  async function completeSession() {
    const previousMode = mode;
    busy = true;
    try {
      const result = await invoke<SessionResultView>("finish_session");
      feedback = result;
      errorMessage = "";
      await setMode("expanded");
      await refreshData(windowKind === "dashboard");
      if (feedbackTimer) clearTimeout(feedbackTimer);
      if (previousMode === "compact") {
        feedbackTimer = setTimeout(() => {
          feedback = null;
          void setMode("compact");
        }, 8000);
      }
    } catch (error) {
      setError(error);
    } finally {
      busy = false;
    }
  }

  async function openDashboard() {
    try {
      await invoke("open_dashboard");
    } catch (error) {
      setError(error);
    }
  }

  async function hideWindow() {
    try {
      await invoke("hide_current_window");
    } catch (error) {
      setError(error);
    }
  }

  function startDrag(event: MouseEvent) {
    if (event.button === 0) void invoke("start_window_drag");
  }

  function setError(error: unknown) {
    errorMessage = error instanceof Error ? error.message : String(error);
  }

  function timerText(totalSeconds: number) {
    const hours = Math.floor(totalSeconds / 3600);
    const minutes = Math.floor((totalSeconds % 3600) / 60);
    const seconds = totalSeconds % 60;
    return hours > 0
      ? `${hours}:${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`
      : `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
  }

  function localizedTitle(title: string) {
    const titles: Record<string, string> = {
      "Novice Learner": "见习学者",
      "Knowledge Hunter": "知识猎手",
      "Scholar Adventurer": "学识冒险家",
      "Master Student": "求学大师",
      "Legendary Learner": "传奇求知者",
    };
    return titles[title] ?? title;
  }

  function localizedQuest(title: string) {
    const minutes = /^Study (\d+) minutes$/.exec(title);
    if (minutes) return `学习 ${minutes[1]} 分钟`;
    const sessions = /^Complete (\d+) study session$/.exec(title);
    if (sessions) return `完成 ${sessions[1]} 次学习`;
    return title;
  }

  function periodEntries(report: StatisticsView): [string, StatisticsPeriodView][] {
    return [
      ["今日", report.today],
      ["本周", report.thisWeek],
      ["本月", report.thisMonth],
      ["累计", report.allTime],
    ];
  }

  function chartHeight(value: number, values: number[]) {
    const maximum = Math.max(...values, 1);
    return Math.max(value === 0 ? 2 : 10, Math.round((value / maximum) * 150));
  }

  function xpPolyline(report: StatisticsView) {
    const values = report.lastSevenDays.map((day) => day.statistics.totalXp);
    const maximum = Math.max(...values, 1);
    const segments = Math.max(1, values.length - 1);
    return values
      .map((value, index) => {
        const x = (index / segments) * 700;
        const y = 145 - (value / maximum) * 125;
        return `${x},${y}`;
      })
      .join(" ");
  }
</script>

{#if !dashboard}
  <main class="loading-shell">
    <div class="loading-orb"></div>
    <span>{errorMessage || "正在唤醒冒险记录…"}</span>
  </main>
{:else if windowKind === "companion"}
  <main class:expanded={mode === "expanded"} class="companion-shell">
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <header class="companion-dragbar" on:mousedown={startDrag}>
      <div class="brand-mark">S</div>
      <div class="brand-copy">
        <strong>Study RPG</strong>
        <span>{dashboard.activeSession ? "专注进行中" : "今日冒险待命"}</span>
      </div>
      <div class="window-actions">
        <button class="icon-button" title="打开完整面板" on:mousedown|stopPropagation on:click={openDashboard}>↗</button>
        <button
          class="icon-button"
          title={mode === "compact" ? "展开" : "收起"}
          on:mousedown|stopPropagation
          on:click={() => setMode(mode === "compact" ? "expanded" : "compact")}
        >{mode === "compact" ? "＋" : "−"}</button>
        <button class="icon-button" title="隐藏到菜单栏" on:mousedown|stopPropagation on:click={hideWindow}>×</button>
      </div>
    </header>

    <section class="hero-row">
      <div
        class="level-ring"
        style={`--xp-progress: ${dashboard.xpProgressPercent * 3.6}deg`}
        aria-label={`等级 ${dashboard.level}，经验进度 ${dashboard.xpProgressPercent}%`}
      >
        <div><small>LV</small><strong>{dashboard.level}</strong></div>
      </div>
      <div class="hero-copy">
        <div class="eyebrow">{localizedTitle(dashboard.title)}</div>
        {#if dashboard.activeSession}
          <strong class="topic-line">{dashboard.activeSession.topic}</strong>
          <span class="timer">{timerText(activeSeconds)}</span>
        {:else}
          <strong class="topic-line">准备开始今天的学习</strong>
          <span class="xp-line">{dashboard.xpIntoLevel} / {dashboard.xpForNextLevel} XP</span>
        {/if}
      </div>
      {#if dashboard.activeSession}
        <button class="primary-button danger" disabled={busy} on:click={completeSession}>结束</button>
      {:else if mode === "compact"}
        <button class="primary-button" on:click={() => setMode("expanded")}>开始</button>
      {/if}
    </section>

    {#if mode === "expanded"}
      <div class="expanded-content">
        {#if !dashboard.activeSession}
          <section class="start-panel">
            <label for="topic">这次准备学习什么？</label>
            <div class="topic-input-row">
              <input
                id="topic"
                bind:value={topic}
                placeholder="例如：Rust ownership"
                on:keydown={(event) => event.key === "Enter" && beginSession()}
              />
              <button class="primary-button" disabled={busy} on:click={() => beginSession()}>开始学习</button>
            </div>
            {#if recentTopics.length > 0}
              <div class="quick-topics">
                {#each recentTopics as recentTopic}
                  <button on:click={() => beginSession(recentTopic)}>{recentTopic}</button>
                {/each}
              </div>
            {/if}
          </section>
        {/if}

        {#if feedback}
          <section class="feedback-card">
            <div class="feedback-spark">✦</div>
            <div>
              <span>冒险结算</span>
              <strong>+{feedback.totalGainedXp} XP</strong>
              <p>{feedback.topic} · {feedback.durationMinutes} 分钟</p>
              {#if feedback.levelAfter > feedback.levelBefore}
                <em>升级！LV {feedback.levelBefore} → LV {feedback.levelAfter}</em>
              {/if}
            </div>
          </section>
        {/if}

        {#if errorMessage}
          <div class="error-banner">{errorMessage}</div>
        {/if}

        <section class="mini-stats">
          <div><span>今日专注</span><strong>{dashboard.todayMinutes}<small> 分钟</small></strong></div>
          <div><span>累计旅程</span><strong>{dashboard.totalSessions}<small> 次</small></strong></div>
          <div><span>当前经验</span><strong>{dashboard.totalXp}<small> XP</small></strong></div>
        </section>

        <section class="quest-panel">
          <div class="section-title">
            <div><span>DAILY QUESTS</span><strong>今日任务</strong></div>
            <button on:click={openDashboard}>查看详情 ↗</button>
          </div>
          <div class="quest-list">
            {#each dashboard.quests as quest}
              <article class:complete={quest.completed} class="quest-item">
                <div class="quest-status">{quest.completed ? "✓" : ""}</div>
                <div class="quest-copy">
                  <div><strong>{localizedQuest(quest.title)}</strong><span>+{quest.rewardXp} XP</span></div>
                  <div class="progress-track">
                    <div style={`width: ${Math.min(100, (quest.current / Math.max(1, quest.target)) * 100)}%`}></div>
                  </div>
                  <small>{quest.current} / {quest.target}</small>
                </div>
              </article>
            {/each}
          </div>
          {#if dashboard.dailyQuestCompleted}
            <div class="all-clear">✦ 今日任务全清 · +{dashboard.dailyQuestRewardXp} XP</div>
          {/if}
        </section>
      </div>
    {:else if errorMessage}
      <div class="compact-error">{errorMessage}</div>
    {/if}
  </main>
{:else}
  <main class="dashboard-shell">
    <aside class="dashboard-sidebar">
      <div class="dashboard-brand"><div class="brand-mark large">S</div><div><strong>Study RPG</strong><span>学习冒险日志</span></div></div>
      <nav>
        <a class="active" href="#overview">◈ 总览</a>
        <a href="#quests">◇ 每日任务</a>
        <a href="#statistics">⌁ 学习统计</a>
      </nav>
      <div class="sidebar-player">
        <span>LV {dashboard.level}</span>
        <strong>{dashboard.playerName}</strong>
        <small>{localizedTitle(dashboard.title)}</small>
      </div>
    </aside>

    <div class="dashboard-content">
      <header id="overview" class="dashboard-header">
        <div><span class="eyebrow">ADVENTURE OVERVIEW</span><h1>欢迎回来，{dashboard.playerName}</h1><p>每一分钟专注，都在塑造更强的你。</p></div>
        <button class="primary-button" on:click={() => invoke("open_companion")}>显示 Companion</button>
      </header>

      {#if errorMessage}<div class="error-banner">{errorMessage}</div>{/if}

      <section class="overview-grid">
        <article class="player-card">
          <div class="level-emblem"><small>LEVEL</small><strong>{dashboard.level}</strong></div>
          <div class="player-progress">
            <span>{localizedTitle(dashboard.title)}</span>
            <strong>{dashboard.xpIntoLevel} / {dashboard.xpForNextLevel} XP</strong>
            <div class="progress-track large-track"><div style={`width: ${dashboard.xpProgressPercent}%`}></div></div>
          </div>
        </article>
        <article class="metric-card"><span>今日学习</span><strong>{dashboard.todayMinutes}</strong><small>分钟</small></article>
        <article class="metric-card"><span>累计 Session</span><strong>{dashboard.totalSessions}</strong><small>次冒险</small></article>
        <article class="metric-card accent"><span>总经验值</span><strong>{dashboard.totalXp}</strong><small>XP</small></article>
      </section>

      <section id="quests" class="dashboard-columns">
        <article class="dashboard-panel">
          <div class="panel-heading"><div><span>DAILY QUESTS</span><h2>今日任务</h2></div><strong>{dashboard.quests.filter((quest) => quest.completed).length}/{dashboard.quests.length}</strong></div>
          <div class="dashboard-quest-list">
            {#each dashboard.quests as quest}
              <div class:complete={quest.completed} class="dashboard-quest">
                <div class="quest-status">{quest.completed ? "✓" : ""}</div>
                <div class="quest-copy"><div><strong>{localizedQuest(quest.title)}</strong><span>+{quest.rewardXp} XP</span></div><div class="progress-track"><div style={`width: ${Math.min(100, (quest.current / Math.max(1, quest.target)) * 100)}%`}></div></div><small>{quest.current} / {quest.target}</small></div>
              </div>
            {/each}
          </div>
        </article>

        <article class="dashboard-panel">
          <div class="panel-heading"><div><span>RECENT RUNS</span><h2>最近学习</h2></div></div>
          <div class="session-list">
            {#if dashboard.recentSessions.length === 0}
              <p class="empty-state">完成第一次学习，开启你的成长旅程。</p>
            {:else}
              {#each dashboard.recentSessions as session}
                <div class="session-row"><div><strong>{session.topic}</strong><small>{session.durationMinutes} 分钟</small></div><span>+{session.earnedXp} XP</span></div>
              {/each}
            {/if}
          </div>
        </article>
      </section>

      {#if statistics}
        <section id="statistics" class="statistics-section">
          <div class="section-heading"><div><span>PROGRESS LOG</span><h2>学习统计</h2></div><div class="streak-pill">🔥 连续 {statistics.currentStreakDays} 天 · 最长 {statistics.longestStreakDays} 天</div></div>
          <div class="period-grid">
            {#each periodEntries(statistics) as [label, period]}
              <article><span>{label}</span><strong>{period.totalMinutes}<small> 分钟</small></strong><p>{period.totalSessions} 次学习 · {period.totalXp} XP</p></article>
            {/each}
          </div>
          <article class="chart-panel">
            <div class="chart-heading"><strong>最近七天</strong><span>学习时长柱 · XP 趋势线</span></div>
            <div class="chart">
              <svg class="xp-line-chart" viewBox="0 0 700 160" preserveAspectRatio="none" aria-label="最近七天 XP 趋势">
                <polyline points={xpPolyline(statistics)}></polyline>
              </svg>
              {#each statistics.lastSevenDays as day}
                <div class="chart-day"><span>{day.statistics.totalMinutes}分</span><div class="bar-wrap"><div class="bar" style={`height: ${chartHeight(day.statistics.totalMinutes, statistics.lastSevenDays.map((item) => item.statistics.totalMinutes))}px`}></div></div><small>{String(day.date.month).padStart(2, "0")}-{String(day.date.day).padStart(2, "0")}</small></div>
              {/each}
            </div>
          </article>
        </section>
      {/if}
    </div>
  </main>
{/if}
