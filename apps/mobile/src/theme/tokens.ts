export const spacing = {
  xs: 4,
  sm: 8,
  md: 12,
  lg: 16,
  xl: 20,
  xxl: 24,
} as const;

export const radius = {
  sm: 8,
  md: 10,
  lg: 12,
  pill: 999,
} as const;

export const typography = {
  largeTitle: { fontSize: 34, fontWeight: "700" as const, letterSpacing: 0.37 },
  title1: { fontSize: 28, fontWeight: "700" as const, letterSpacing: 0.36 },
  title2: { fontSize: 22, fontWeight: "700" as const, letterSpacing: 0.35 },
  title3: { fontSize: 20, fontWeight: "600" as const, letterSpacing: 0.38 },
  headline: { fontSize: 17, fontWeight: "600" as const, letterSpacing: -0.41 },
  body: { fontSize: 17, fontWeight: "400" as const, letterSpacing: -0.41 },
  callout: { fontSize: 16, fontWeight: "400" as const, letterSpacing: -0.32 },
  subhead: { fontSize: 15, fontWeight: "400" as const, letterSpacing: -0.24 },
  footnote: { fontSize: 13, fontWeight: "400" as const, letterSpacing: -0.08 },
  caption1: { fontSize: 12, fontWeight: "400" as const, letterSpacing: 0 },
  caption2: { fontSize: 11, fontWeight: "600" as const, letterSpacing: 0.06 },
  sectionHeader: {
    fontSize: 13,
    fontWeight: "400" as const,
    letterSpacing: -0.08,
  },
} as const;

export const shadow = {
  card: "0 1px 3px rgba(0, 0, 0, 0.06), 0 1px 2px rgba(0, 0, 0, 0.04)",
  elevated: "0 4px 12px rgba(0, 0, 0, 0.08)",
} as const;
