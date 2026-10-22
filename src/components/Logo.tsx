/** Cipherleaf mark: a leaf whose vein forms a keyhole. */
export default function Logo({ size = 22 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 32 32" fill="none" aria-hidden>
      <defs>
        <linearGradient id="cl-leaf" x1="4" y1="28" x2="28" y2="4" gradientUnits="userSpaceOnUse">
          <stop stopColor="var(--accent)" />
          <stop offset="1" stopColor="var(--accent-strong)" />
        </linearGradient>
      </defs>
      <path
        d="M27 5C15 5 5 11.5 5 22.5c0 2 .4 3.4.9 4.5C7 25 9.5 21.5 13 19c-3 3.4-5 6.6-6.1 9C21 27 27 18 27 5Z"
        fill="url(#cl-leaf)"
      />
      <circle cx="18.2" cy="13.2" r="2.6" fill="var(--bg)" />
      <path d="M17.3 14.6h1.8l.9 4.4h-3.6l.9-4.4Z" fill="var(--bg)" />
    </svg>
  );
}
