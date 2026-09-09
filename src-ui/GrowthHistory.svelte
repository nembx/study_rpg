<script lang="ts">
  import { onMount } from "svelte";
  import { skillPathLabels } from "./skillLabels";
  import type { GrowthEventView, GrowthHistoryPageView, SkillProgressView } from "./types";

  export let skills: SkillProgressView[];
  export let latestEventId: number | null;
  export let loadHistory: (skillId: number | null, beforeId: number | null) => Promise<GrowthHistoryPageView>;

  let selectedSkill = "";
  let events: GrowthEventView[] = [];
  let nextBeforeId: number | null = null;
  let seenLatestId: number | null = null;
  let retryBeforeId: number | null = null;
  let loading = false;
  let loaded = false;
  let errorMessage = "";
  let requestVersion = 0;

  $: labels = skillPathLabels(skills);
  $: newEventsAvailable = loaded && latestEventId !== seenLatestId;

  onMount(() => {
    void loadPage();
    return () => { requestVersion += 1; };
  });

  async function loadPage(beforeId: number | null = null) {
    const version = ++requestVersion;
    const latestAtRequest = latestEventId;
    loading = true;
    errorMessage = "";
    retryBeforeId = beforeId;
    if (beforeId === null) {
      events = [];
      nextBeforeId = null;
      loaded = false;
    }
    try {
      const page = await loadHistory(selectedSkill ? Number(selectedSkill) : null, beforeId);
      if (version !== requestVersion) return;
      events = beforeId === null ? page.events : [...events, ...page.events];
      nextBeforeId = page.nextBeforeId;
      loaded = true;
      if (beforeId === null) seenLatestId = latestAtRequest;
    } catch (error) {
      if (version === requestVersion) {
        errorMessage = error instanceof Error ? error.message : String(error);
      }
    } finally {
      if (version === requestVersion) loading = false;
    }
  }

  function changeSkill(value: string) {
    selectedSkill = value;
    void loadPage();
  }

  function growthTime(epochSeconds: number | null) {
    if (epochSeconds === null) return "手动结算";
    return new Intl.DateTimeFormat("zh-CN", {
      month: "2-digit", day: "2-digit", hour: "2-digit", minute: "2-digit", hour12: false,
    }).format(new Date(epochSeconds * 1000));
  }
</script>

<article id="growth-history" class="dashboard-panel growth-history-panel" aria-busy={loading}>
  <div class="panel-heading">
    <div><span>GROWTH HISTORY</span><h2>成长历史</h2></div>
    <button class="history-button" disabled={loading} on:click={() => loadPage()}>刷新记录</button>
  </div>

  <div class="history-filter">
    <label for="growth-skill-filter">筛选技能</label>
    <select
      id="growth-skill-filter"
      class="skill-select"
      bind:value={selectedSkill}
      disabled={loading}
      on:change={(event) => changeSkill(event.currentTarget.value)}
    >
      <option value="">全部成长记录</option>
      {#each skills as skill (skill.id)}
        <option value={String(skill.id)}>{labels.get(skill.id)}</option>
      {/each}
    </select>
  </div>

  {#if newEventsAvailable}
    <button class="new-growth-notice" disabled={loading} on:click={() => loadPage()}>有新的成长记录 · 点击刷新</button>
  {/if}

  {#if errorMessage}
    <div class="history-error" role="alert">
      <span>加载成长记录失败：{errorMessage}</span>
      <button class="history-button" disabled={loading} on:click={() => loadPage(retryBeforeId)}>重试</button>
    </div>
  {/if}

  {#if events.length > 0}
    <div class="growth-timeline" aria-label="成长事件">
      {#each events as event (event.id)}
        <article class:level-event={event.details.kind === "playerLevelChange"} class="growth-event">
          <div class="growth-event-icon" aria-hidden="true">{event.details.kind === "playerLevelChange" ? "↑" : "✦"}</div>
          <div class="growth-event-copy">
            <div class="growth-event-meta"><span>{event.details.kind === "playerLevelChange" ? "LEVEL CHANGE" : "SKILL GROWTH"}</span><time>{growthTime(event.occurredAtEpochSeconds)}</time></div>
            {#if event.details.kind === "playerLevelChange"}
              <strong>角色升级 · LV {event.details.levelBefore} → LV {event.details.levelAfter}</strong>
              <p>{event.topic} · +{event.details.gainedXp} XP · 累计 {event.details.totalXpAfter} XP</p>
            {:else}
              <strong>{event.details.skillName} · +{event.details.gainedXp} XP</strong>
              <p>{event.topic} · {event.details.levelAfter > event.details.levelBefore ? `技能升级 LV ${event.details.levelBefore} → LV ${event.details.levelAfter}` : `累计 ${event.details.totalXpAfter} XP`}</p>
            {/if}
          </div>
        </article>
      {/each}
    </div>
  {:else if loaded && !loading && !errorMessage}
    <p class="empty-state">{selectedSkill ? "这个技能还没有成长记录。" : "还没有成长记录。技能获得 XP 或角色升级后会保存在这里。"}</p>
  {/if}

  <div class="history-footer">
    <p role="status">{loading ? "正在加载成长记录…" : loaded ? `已显示 ${events.length} 条` : ""}</p>
    {#if nextBeforeId !== null && !errorMessage}
      <button class="history-button" disabled={loading} on:click={() => loadPage(nextBeforeId)}>加载更早记录</button>
    {:else if loaded && events.length > 0 && !loading && !errorMessage}
      <span>已到最早的记录</span>
    {/if}
  </div>
</article>

<style>
  .growth-history-panel { min-width: 0; scroll-margin-top: 20px; }
  .history-button { flex-shrink: 0; border: 1px solid var(--line); border-radius: 8px; padding: 8px 10px; color: var(--accent-bright); background: rgba(159, 122, 234, .07); font-size: 10px; }
  .history-button:hover { border-color: var(--accent); background: rgba(159, 122, 234, .16); }
  button:focus-visible { outline: 2px solid var(--cyan); outline-offset: 3px; }
  button:disabled { opacity: .55; cursor: wait; }
  .history-filter { display: flex; flex-direction: column; gap: 7px; margin-top: 15px; }
  .history-filter label { color: var(--muted); font-size: 10px; }
  .history-filter select { width: 100%; min-width: 0; min-height: 36px; border: 1px solid var(--line); border-radius: 9px; padding: 9px 28px 9px 11px; outline: 0; color: #f6f3ff; background-color: #111626; font-size: 11px; color-scheme: dark; }
  .history-filter select:focus { border-color: var(--cyan); box-shadow: 0 0 0 3px rgba(103, 216, 210, .1); }
  .new-growth-notice { width: 100%; margin-top: 12px; border: 1px solid rgba(103, 216, 210, .25); border-radius: 9px; padding: 10px; color: var(--cyan); background: rgba(103, 216, 210, .07); font-size: 10px; text-align: left; }
  .history-error { display: flex; align-items: center; justify-content: space-between; gap: 10px; margin-top: 12px; padding: 10px; border: 1px solid rgba(255, 127, 147, .25); border-radius: 9px; color: var(--danger); font-size: 10px; line-height: 1.6; }
  .history-error span { min-width: 0; overflow-wrap: anywhere; }
  .history-footer { display: flex; align-items: center; justify-content: space-between; gap: 10px; margin-top: 14px; color: var(--muted); font-size: 9px; }
  .history-footer p { margin: 0; }
  .empty-state { line-height: 1.7; }
</style>
