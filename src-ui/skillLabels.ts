import type { SkillProgressView } from "./types";

// Dashboard supplies skills in parent-first order.
export function skillPathLabels(skills: SkillProgressView[]): Map<number, string> {
  const labels = new Map<number, string>();
  for (const skill of skills) {
    const parent = skill.parentId === null ? undefined : labels.get(skill.parentId);
    labels.set(skill.id, parent ? `${parent} / ${skill.name}` : skill.name);
  }
  return labels;
}
