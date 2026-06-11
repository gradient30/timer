export type AppTheme = "dark" | "light" | "vivid";

const THEME_ORDER: AppTheme[] = ["dark", "light", "vivid"];

export function normalizeTheme(value: string | undefined | null): AppTheme {
  return THEME_ORDER.includes(value as AppTheme) ? (value as AppTheme) : "dark";
}

export function applyTheme(theme: AppTheme) {
  document.documentElement.dataset.theme = theme;
}

export function getNextTheme(theme: AppTheme): AppTheme {
  const index = THEME_ORDER.indexOf(theme);
  return THEME_ORDER[(index + 1) % THEME_ORDER.length];
}

export function getThemeToggleLabel(theme: AppTheme): string {
  const next = getNextTheme(theme);
  if (next === "light") return "明";
  if (next === "vivid") return "彩";
  return "暗";
}

export function getThemeToggleTitle(theme: AppTheme): string {
  const next = getNextTheme(theme);
  if (next === "light") return "切换为明亮风格";
  if (next === "vivid") return "切换为彩色风格";
  return "切换为深色风格";
}
