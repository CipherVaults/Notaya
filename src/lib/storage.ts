// Small, non-secret choices kept in the browser profile (look, layout, dismissed notes).
//
// Until V1.3.0 the keys started with "cipherleaf."; since V1.4.0 they start with "notaya.". A value saved
// under the old name is read once and copied to the new name, so nobody loses their look or layout. The old
// entry is left where it is (older versions still read it). Callers wrap these in try/catch, because browser
// storage can be unavailable.

const PREFIX = "notaya.";
const OLD_PREFIX = "cipherleaf.";

export function getKey(name: string): string | null {
  const current = localStorage.getItem(PREFIX + name);
  if (current !== null) return current;
  const old = localStorage.getItem(OLD_PREFIX + name);
  if (old !== null) {
    try {
      localStorage.setItem(PREFIX + name, old);
    } catch {
      // Not critical: the old value is used again next time.
    }
  }
  return old;
}

export function setKey(name: string, value: string): void {
  localStorage.setItem(PREFIX + name, value);
}
