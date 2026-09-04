import type { CharacterClassId } from "./types";

export interface CharacterClassMeta {
  id: CharacterClassId;
  name: string;
  description: string;
}

export const CHARACTER_CLASSES = [
  { id: "scholar", name: "学者", description: "以知识积累推动稳定成长" },
  { id: "engineer", name: "工程师", description: "把复杂目标拆成可执行系统" },
  { id: "mage", name: "法师", description: "在专注中驾驭灵感与创造力" },
  { id: "warrior", name: "战士", description: "依靠纪律完成每日训练" },
  { id: "archer", name: "游侠", description: "瞄准目标并保持轻快节奏" },
] as const satisfies readonly CharacterClassMeta[];

export function normalizeCharacterClassId(value: string): CharacterClassId {
  return CHARACTER_CLASSES.find((item) => item.id === value)?.id ?? "scholar";
}

export function characterClassName(characterClass: string): string {
  const resolvedClass = normalizeCharacterClassId(characterClass);
  return CHARACTER_CLASSES.find((item) => item.id === resolvedClass)?.name ?? "学者";
}
