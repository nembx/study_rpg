<script lang="ts">
  import type { CharacterClassId } from "./types";
  import { normalizeCharacterClassId } from "./characterClasses";

  export let characterClass: CharacterClassId = "scholar";
  export let size: "tiny" | "small" | "medium" | "large" = "medium";
  export let mood: "idle" | "focus" | "celebrate" = "idle";
  export let label = "";
  export let decorative = false;

  const palettes: Record<
    CharacterClassId,
    {
      primary: string;
      secondary: string;
      shadow: string;
      skin: string;
      hair: string;
      accent: string;
      background: string;
    }
  > = {
    scholar: {
      primary: "#8e79ed",
      secondary: "#63d8d0",
      shadow: "#30275f",
      skin: "#f1c3a6",
      hair: "#302743",
      accent: "#ffd37c",
      background: "#171b3a",
    },
    engineer: {
      primary: "#48b8c5",
      secondary: "#a7e36e",
      shadow: "#164c64",
      skin: "#eebc9d",
      hair: "#263b4d",
      accent: "#ffd37c",
      background: "#102a3a",
    },
    mage: {
      primary: "#bf78ed",
      secondary: "#f38fbc",
      shadow: "#572a75",
      skin: "#f2c4b1",
      hair: "#39214f",
      accent: "#ffe48f",
      background: "#24183e",
    },
    warrior: {
      primary: "#ed7d73",
      secondary: "#f4bd66",
      shadow: "#713743",
      skin: "#eab895",
      hair: "#3b2733",
      accent: "#ffe09a",
      background: "#351b2a",
    },
    archer: {
      primary: "#6fc58d",
      secondary: "#7fc5e8",
      shadow: "#245345",
      skin: "#efc4a0",
      hair: "#263b38",
      accent: "#f6d477",
      background: "#152f32",
    },
  };

  $: visualClass = normalizeCharacterClassId(characterClass);
  $: palette = palettes[visualClass];
  $: avatarLabel = label || "角色立绘";
  $: avatarStyle = `--avatar-primary: ${palette.primary}; --avatar-secondary: ${palette.secondary}; --avatar-shadow: ${palette.shadow}; --avatar-skin: ${palette.skin}; --avatar-hair: ${palette.hair}; --avatar-accent: ${palette.accent}; --avatar-background: ${palette.background};`;
</script>

<div
  class={`character-avatar ${size} ${mood}`}
  style={avatarStyle}
  role={decorative ? undefined : "img"}
  aria-label={decorative ? undefined : avatarLabel}
  aria-hidden={decorative ? "true" : undefined}
>
  <svg viewBox="0 0 120 144" aria-hidden="true" shape-rendering="crispEdges">
    {#if !decorative}<title>{avatarLabel}</title>{/if}

    <rect class="avatar-backplate" x="5" y="5" width="110" height="134" rx="18" />
    <path class="avatar-frame" d="M20 5h18v3H20zM82 5h18v3H82zM5 24h3v18H5zM112 24h3v18h-3zM5 102h3v18H5zM112 102h3v18h-3z" />
    <g class="avatar-stars" fill="var(--avatar-secondary)">
      <rect x="19" y="25" width="3" height="3" />
      <rect x="97" y="29" width="3" height="3" />
      <rect x="89" y="91" width="2" height="2" opacity=".7" />
      <rect x="26" y="96" width="2" height="2" opacity=".7" />
    </g>

    <ellipse class="avatar-ground" cx="60" cy="127" rx="32" ry="6" />

    <g class="avatar-body">
      <path class="avatar-cape" d="M32 88h56l8 36H24z" />
      <path class="avatar-cape-highlight" d="M39 91h12v31H34z" />
      <rect class="avatar-torso" x="42" y="82" width="36" height="40" rx="4" />
      <rect class="avatar-belt" x="40" y="103" width="40" height="7" />
      <rect class="avatar-buckle" x="57" y="104" width="7" height="5" />
      <path class="avatar-arm" d="M39 86h-8v27h12v-9h-4zM81 86h8v27H77v-9h4z" />
      <path class="avatar-leg" d="M45 116h13v10H42zM62 116h13v10H78l-3 4H61z" />
      <rect class="avatar-boot" x="39" y="126" width="19" height="5" />
      <rect class="avatar-boot" x="63" y="126" width="19" height="5" />

      <path class="avatar-neck" d="M52 75h16v12H52z" />
      <path class="avatar-face" d="M38 46h44v29l-8 9H47l-9-9z" />
      <path class="avatar-hair" d="M35 48h8V39h34v5h9v24h-8V57H44v16h-9z" />
      <rect class="avatar-hair-shine" x="43" y="42" width="12" height="5" />
      <rect class="avatar-eye" x="48" y="62" width="5" height="5" />
      <rect class="avatar-eye" x="67" y="62" width="5" height="5" />
      <rect class="avatar-cheek" x="43" y="70" width="5" height="3" />
      <rect class="avatar-cheek" x="72" y="70" width="5" height="3" />
      <rect class="avatar-mouth" x="56" y="73" width="9" height="3" />
    </g>

    {#if visualClass === "scholar"}
      <g class="avatar-prop scholar-prop">
        <path d="M22 91h17v19H22z" fill="var(--avatar-accent)" />
        <path d="M30 91h9v19h-9z" fill="#fff0ba" />
        <path d="M23 94h14M23 98h12M23 102h14" stroke="var(--avatar-shadow)" stroke-width="2" />
        <rect x="91" y="57" width="4" height="4" fill="var(--avatar-accent)" />
        <rect x="98" y="50" width="3" height="3" fill="var(--avatar-secondary)" />
        <rect x="92" y="45" width="2" height="2" fill="var(--avatar-accent)" />
      </g>
    {:else if visualClass === "engineer"}
      <g class="avatar-prop engineer-prop">
        <path d="M43 60h34v9H43z" fill="var(--avatar-accent)" />
        <rect x="47" y="62" width="10" height="5" fill="var(--avatar-shadow)" />
        <rect x="63" y="62" width="10" height="5" fill="var(--avatar-shadow)" />
        <rect x="57" y="63" width="7" height="3" fill="var(--avatar-secondary)" />
        <path d="M88 94h15v15H88z" fill="var(--avatar-secondary)" />
        <path d="M95 91v21M91 98h8M91 105h8" stroke="var(--avatar-shadow)" stroke-width="2" />
      </g>
    {:else if visualClass === "mage"}
      <g class="avatar-prop mage-prop">
        <path d="M34 42h52l-8-11H42z" fill="var(--avatar-primary)" />
        <rect x="57" y="21" width="6" height="10" fill="var(--avatar-accent)" />
        <rect x="54" y="18" width="12" height="4" fill="var(--avatar-secondary)" />
        <path d="M92 76h4v37h-4z" fill="var(--avatar-accent)" />
        <path d="M94 67l5 8-5 8-5-8z" fill="var(--avatar-secondary)" />
        <rect x="98" y="64" width="3" height="3" fill="var(--avatar-accent)" />
      </g>
    {:else if visualClass === "warrior"}
      <g class="avatar-prop warrior-prop">
        <path d="M39 47h42v10H39z" fill="var(--avatar-primary)" />
        <path d="M47 39h26v8H47z" fill="var(--avatar-secondary)" />
        <path d="M93 47h5v55h-5z" fill="#d5e7ec" />
        <path d="M87 54h17v6H87z" fill="var(--avatar-accent)" />
        <path d="M90 44h11l-5-8z" fill="#eaf5f5" />
      </g>
    {:else}
      <g class="avatar-prop archer-prop">
        <path d="M89 47q19 26 0 52" fill="none" stroke="var(--avatar-accent)" stroke-width="4" />
        <path d="M91 49l-20 25 20 23" fill="none" stroke="var(--avatar-secondary)" stroke-width="2" />
        <path d="M28 83h8v28h-8z" fill="var(--avatar-shadow)" />
        <path d="M23 82h18v6H23z" fill="var(--avatar-accent)" />
        <rect x="96" y="38" width="3" height="3" fill="var(--avatar-secondary)" />
      </g>
    {/if}

    {#if mood === "focus"}
      <g class="avatar-focus-mark" fill="var(--avatar-secondary)">
        <rect x="17" y="58" width="3" height="12" />
        <rect x="14" y="61" width="9" height="3" />
        <rect x="99" y="110" width="3" height="12" />
        <rect x="96" y="113" width="9" height="3" />
      </g>
    {:else if mood === "celebrate"}
      <g class="avatar-celebrate-mark" fill="var(--avatar-accent)">
        <rect x="16" y="52" width="4" height="4" />
        <rect x="101" y="68" width="4" height="4" />
        <path d="M23 42h4v12h-4zM19 46h12v4H19z" />
        <path d="M94 86h4v12h-4zM90 90h12v4H90z" />
      </g>
    {/if}
  </svg>
</div>

<style>
  .character-avatar {
    position: relative;
    display: inline-flex;
    flex: 0 0 auto;
    width: 64px;
    aspect-ratio: 5 / 6;
    isolation: isolate;
    filter: drop-shadow(0 8px 12px rgba(0, 0, 0, 0.26));
  }

  .character-avatar::after {
    content: "";
    position: absolute;
    z-index: -1;
    inset: 18% 12% 7%;
    border-radius: 40%;
    background: var(--avatar-secondary);
    opacity: 0.12;
    filter: blur(13px);
  }

  svg {
    display: block;
    width: 100%;
    height: 100%;
    overflow: visible;
    image-rendering: pixelated;
  }

  .tiny { width: 30px; }
  .small { width: 40px; }
  .medium { width: 72px; }
  .large { width: 132px; }
  .focus::after { opacity: 0.22; }
  .focus svg { transform: translateY(-1px) scale(1.015); }
  .celebrate::after { opacity: 0.3; }
  .celebrate svg { transform: translateY(-3px); }
  .avatar-backplate { fill: var(--avatar-background); stroke: var(--avatar-primary); stroke-width: 2; }
  .avatar-frame { fill: var(--avatar-secondary); opacity: 0.8; }
  .avatar-ground { fill: var(--avatar-shadow); opacity: 0.65; }
  .avatar-cape { fill: var(--avatar-shadow); }
  .avatar-cape-highlight { fill: var(--avatar-primary); opacity: 0.92; }
  .avatar-torso { fill: var(--avatar-primary); }
  .avatar-belt { fill: var(--avatar-shadow); }
  .avatar-buckle { fill: var(--avatar-accent); }
  .avatar-arm { fill: var(--avatar-primary); }
  .avatar-leg, .avatar-boot { fill: var(--avatar-shadow); }
  .avatar-neck, .avatar-face { fill: var(--avatar-skin); }
  .avatar-hair, .avatar-hair-shine { fill: var(--avatar-hair); }
  .avatar-hair-shine { opacity: 0.58; }
  .avatar-eye { fill: #202035; }
  .avatar-cheek { fill: #e68e8c; opacity: 0.7; }
  .avatar-mouth { fill: #a65365; }
  .avatar-prop { filter: drop-shadow(0 1px 0 rgba(0, 0, 0, 0.22)); }
  .avatar-focus-mark, .avatar-celebrate-mark { opacity: 0.9; }
</style>
