const PATHS = {
  plus: "M12 5v14M5 12h14",
  sliders: "M4 7h9M17 7h3M4 17h3M11 17h9M15 4.5v5M9 14.5v5",
  expand: "M4 9V4h5M20 9V4h-5M4 15v5h5M20 15v5h-5",
  shrink: "M9 4v5H4M15 4v5h5M9 20v-5H4M15 20v-5h5",
  search: "M11 18a7 7 0 1 0 0-14 7 7 0 0 0 0 14zM21 21l-4.5-4.5",
  play: "M8 5v14l11-7z",
  star: "M12 3.5l2.6 5.4 5.9.8-4.3 4.1 1 5.9L12 16.9 6.8 19.7l1-5.9L3.5 9.7l5.9-.8z",
  info: "M12 11v6M12 7.5v.01M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0z",
  pencil: "M4 20h4L19 9l-4-4L4 16zM13.5 6.5l4 4",
  close: "M6 6l12 12M18 6L6 18",
  layers: "M12 4l8 4-8 4-8-4zM4 12l8 4 8-4M4 16l8 4 8-4",
  skip: "M6 7l5 5-5 5M13 7l5 5-5 5",
  check: "M5 12.5l4.5 4.5L19 7",
  clock: "M12 7v5l3 2M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0z",
  ban: "M5.6 5.6l12.8 12.8M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0z",
  list: "M5 7h14M5 12h14M5 17h9",
  chart: "M5 20V10M12 20V4M19 20v-7",
  download: "M12 4v10M7.5 10l4.5 4.5 4.5-4.5M5 19h14",
  pause: "M8 5v14M16 5v14",
  trash: "M5 7h14M10 4h4M8 7l1 12h6l1-12",
  refresh: "M20 11a8 8 0 1 0-2.3 5.7M20 5v6h-6",
  store: "M5 8h14l-1 12H6zM9 8V6a3 3 0 0 1 6 0v2",
} as const;

export type IconName = keyof typeof PATHS;

export function Icon({ name, size = 18, filled = false }: { name: IconName; size?: number; filled?: boolean }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill={filled || name === "play" ? "currentColor" : "none"}
      stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      <path d={PATHS[name]} />
    </svg>
  );
}
