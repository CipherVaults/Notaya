/** Validate an editor draft and carry its decimal digits unchanged to the save command. */
export function decimalInput(draft: string): string | null {
  const token = draft.trim().replace(/,/g, "");
  if (token === "") return null;
  const decimal = /^[+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:e[+-]?\d+)?$/i;
  if (!decimal.test(token) || !Number.isFinite(Number(token))) {
    throw new Error("Enter a finite decimal number.");
  }
  return token;
}
