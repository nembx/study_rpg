<script lang="ts">
  import { tick } from "svelte";
  import { skillPathLabels } from "./skillLabels";
  import type { SkillProgressView } from "./types";

  export let skills: SkillProgressView[];
  export let oncreate: (name: string, parentId: number | null) => Promise<void>;

  let showForm = false;
  let name = "";
  let parentId: number | null = null;
  let saving = false;
  let errorMessage = "";
  let successMessage = "";
  let nameInput: HTMLInputElement | undefined;

  $: labels = skillPathLabels(skills);

  async function openForm(parent: number | null) {
    parentId = parent;
    name = "";
    errorMessage = "";
    successMessage = "";
    showForm = true;
    await tick();
    nameInput?.focus();
  }

  async function createSkill() {
    if (saving || !name.trim()) return;
    saving = true;
    errorMessage = "";
    try {
      await oncreate(name.trim(), parentId);
      successMessage = `已创建「${name.trim()}」`;
      showForm = false;
      name = "";
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : String(error);
    } finally {
      saving = false;
    }
  }
</script>

<article id="skills" class="dashboard-panel skill-tree-panel">
  <div class="panel-heading">
    <div><span>STUDY SKILLS · {skills.length}</span><h2>技能树</h2></div>
    <button class="new-skill-button" disabled={saving} on:click={() => openForm(null)}>＋ 新建技能</button>
  </div>
  <p class="skill-tree-intro">组织你的学习方向，在 Companion 中选择本次成长的技能。</p>

  {#if showForm}
    <form class="skill-create-form" on:submit|preventDefault={createSkill}>
      <label for="new-skill-name">技能名称</label>
      <input
        id="new-skill-name"
        bind:this={nameInput}
        bind:value={name}
        placeholder="例如：编程、Rust、英语"
        disabled={saving}
        required
      />
      <label for="parent-skill">所属技能</label>
      <select id="parent-skill" class="skill-select" bind:value={parentId} disabled={saving}>
        <option value={null}>无，创建根技能</option>
        {#each skills as skill (skill.id)}
          <option value={skill.id}>{labels.get(skill.id)}</option>
        {/each}
      </select>
      <p class="skill-form-hint">同一父技能下的名称不能重复。</p>
      {#if errorMessage}<p class="skill-form-error" role="alert">{errorMessage}</p>{/if}
      <div class="skill-form-actions">
        <button type="button" class="cancel-skill-button" disabled={saving} on:click={() => showForm = false}>取消</button>
        <button type="submit" class="primary-button" disabled={saving || !name.trim()}>{saving ? "正在保存…" : "创建技能"}</button>
      </div>
    </form>
  {/if}

  {#if successMessage}<p class="skill-form-success" role="status">{successMessage}</p>{/if}

  {#if skills.length === 0}
    <div class="skill-tree-empty">
      <span aria-hidden="true">✦</span>
      <strong>从一个学习方向开始</strong>
      <p>创建根技能，再为它添加细分能力。每次学习都会留下成长。</p>
    </div>
  {:else}
    <ul class="skill-progress-list skill-tree-list" aria-label="学习技能">
      {#each skills as skill (skill.id)}
        <li
          class="skill-progress-row"
          class:child-skill={skill.depth > 0}
          style={`--skill-depth: ${Math.min(skill.depth, 4)}`}
        >
          {#if skill.parentId !== null && labels.has(skill.parentId)}
            <p class="skill-ancestry" title={labels.get(skill.parentId)}>{labels.get(skill.parentId)}</p>
          {/if}
          <div class="skill-progress-heading">
            <div><strong title={labels.get(skill.id)}>{skill.name}</strong><span>LV {skill.level}</span></div>
            <em>{skill.totalXp} XP</em>
          </div>
          <div class="progress-track"><div style={`width: ${skill.xpProgressPercent}%`}></div></div>
          <div class="skill-progress-meta"><span>{skill.xpIntoLevel} / {skill.xpForNextLevel} XP</span><span>掌握度 {skill.masteryPercent}%</span></div>
          <button
            class="add-child-button"
            disabled={saving}
            aria-label={`为 ${labels.get(skill.id)} 添加子技能`}
            on:click={() => openForm(skill.id)}
          >＋ 子技能</button>
        </li>
      {/each}
    </ul>
  {/if}
</article>

<style>
  .skill-tree-panel { min-width: 0; scroll-margin-top: 20px; }
  .skill-tree-intro { margin: 12px 0 0; color: var(--muted); font-size: 10px; line-height: 1.6; }
  .new-skill-button, .cancel-skill-button, .add-child-button { border: 1px solid var(--line); border-radius: 8px; color: var(--accent-bright); background: rgba(159, 122, 234, .07); font-size: 10px; }
  .new-skill-button { flex-shrink: 0; padding: 8px 10px; }
  .new-skill-button:hover, .add-child-button:hover { border-color: var(--accent); background: rgba(159, 122, 234, .16); }
  button:focus-visible { outline: 2px solid var(--cyan); outline-offset: 3px; }
  button:disabled { opacity: .55; cursor: wait; }
  .skill-create-form { display: flex; flex-direction: column; gap: 7px; margin-top: 14px; padding: 14px; border: 1px solid rgba(196, 167, 255, .28); border-radius: 12px; background: rgba(5, 8, 16, .34); }
  .skill-create-form label { color: #d8d8e7; font-size: 10px; font-weight: 700; }
  .skill-create-form label:not(:first-child) { margin-top: 4px; }
  .skill-create-form input, .skill-create-form select { width: 100%; min-width: 0; border: 1px solid var(--line); border-radius: 8px; padding: 9px; outline: 0; color: #f6f3ff; background-color: #111626; font: inherit; font-size: 11px; }
  .skill-create-form input:focus, .skill-create-form select:focus { border-color: var(--cyan); box-shadow: 0 0 0 3px rgba(103, 216, 210, .1); }
  .skill-create-form select { min-height: 36px; padding-right: 28px; color-scheme: dark; }
  .skill-form-hint { margin: 0; color: var(--muted); font-size: 9px; line-height: 1.5; }
  .skill-form-actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 4px; }
  .cancel-skill-button { padding: 8px 12px; color: var(--muted); }
  .skill-form-error, .skill-form-success { margin: 5px 0 0; font-size: 10px; line-height: 1.6; overflow-wrap: anywhere; }
  .skill-form-error { color: var(--danger); }
  .skill-form-success { margin-top: 12px; color: var(--success); }
  .skill-tree-empty { display: flex; align-items: center; flex-direction: column; padding: 28px 12px; text-align: center; }
  .skill-tree-empty > span { margin-bottom: 10px; color: var(--accent-bright); font-size: 28px; }
  .skill-tree-empty strong { font-size: 12px; }
  .skill-tree-empty p { max-width: 250px; margin: 8px 0 0; color: var(--muted); font-size: 10px; line-height: 1.7; }
  .skill-tree-list { padding: 0; list-style: none; }
  .skill-progress-row { position: relative; min-width: 0; margin-left: calc(var(--skill-depth) * 14px); }
  .child-skill::before { position: absolute; top: -9px; left: -10px; width: 9px; height: 25px; border-left: 1px solid rgba(159, 122, 234, .35); border-bottom: 1px solid rgba(159, 122, 234, .35); border-radius: 0 0 0 5px; content: ""; }
  .skill-ancestry { margin: 0 0 6px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--muted); font-size: 8px; }
  .skill-progress-heading { gap: 8px; }
  .skill-progress-heading > div { min-width: 0; flex-wrap: wrap; }
  .skill-progress-heading strong { overflow-wrap: anywhere; }
  .skill-progress-heading em { flex-shrink: 0; }
  .skill-progress-meta { flex-wrap: wrap; gap: 4px; }
  .add-child-button { margin-top: 9px; padding: 5px 8px; font-size: 9px; }
</style>
