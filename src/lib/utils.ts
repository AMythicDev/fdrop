export function filename(s: string): string {
  let last_fwdslash = s.lastIndexOf("/") + 1;
  return s.slice(last_fwdslash, s.length);
}
