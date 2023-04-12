export const padZeros = (v: string, n: number) =>
  "0".repeat(Math.max(0, n - v.length)) + v;

export const toHexString = (v: number) => v.toString(16);

export const vidPidString = (vid: number, pid: number) =>
  padZeros(toHexString(vid), 4) + ":" + padZeros(toHexString(pid), 4);

export const sleep = (ms: number) =>
  new Promise((resolve) => setTimeout(resolve, ms));

// Assertion for type-checked matching of all variants in sum types, e.g.
// put it in the unreachable else branch when matching a sum type using if-else chain.
export const assertUnreachable = (_x: never): never => {
  throw new Error("Unhandled variant");
};
