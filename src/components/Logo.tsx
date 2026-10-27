/** Notaya mark: two sticky notes, amber behind and coral in front (the Sunrise colours). */
export default function Logo({ size = 22 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 32 32" fill="none" aria-hidden>
      <rect x="3.5" y="4.5" width="15.5" height="15.5" rx="1.4" fill="#ffb020" transform="rotate(-8 11 12)" />
      <g transform="rotate(6 19 19)">
        <path d="M11.5 11.5h16v11.2L23.2 27H11.5Z" fill="#ff6b5a" />
        <path d="M27.5 22.7 23.2 27v-3.3c0-.6.4-1 1-1Z" fill="#d9483a" />
        <rect x="14" y="15" width="10" height="1.6" rx=".8" fill="#ffffff" opacity=".75" />
        <rect x="14" y="18.6" width="7.5" height="1.6" rx=".8" fill="#ffffff" opacity=".75" />
      </g>
    </svg>
  );
}
