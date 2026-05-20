export type ConsoleSegment = { kind: "plain" | "error"; text: string };

function trimTrailingEmpty(lines: string[]): string[] {
  let end = lines.length;
  while (end > 0 && lines[end - 1].trim() === "") end--;
  return lines.slice(0, end);
}

function normalizeForDetection(line: string): string {
  return line.trimStart().replace(/^\[\d+\]\s?/, "");
}

function isExplicitErrorBlockStart(line: string): boolean {
  const n = normalizeForDetection(line).toLowerCase();
  return n.includes("==== begin error ====");
}

function isExplicitErrorBlockEnd(line: string): boolean {
  const n = normalizeForDetection(line).toLowerCase();
  return n.includes("==== end error ====");
}

function isErrorStarter(line: string): boolean {
  const s = normalizeForDetection(line).trimStart();
  if (!s) return false;
  const lower = s.toLowerCase();
  if (lower.startsWith("npm err!")) return true;
  if (/^error(\s|:)/i.test(s)) return true;
  if (/^⨯\s/.test(s)) return true;
  if (/^trace:\s*error\b/i.test(s)) return true;
  if (lower.includes("severity: 'error'") || lower.includes('severity: "error"')) return true;
  if (lower.includes("unhandledpromiserejection")) return true;
  if (lower.includes("uncaught exception")) return true;
  if (/^\[error\]/i.test(s)) return true;
  if (/^\s*error\s*:/i.test(s)) return true;
  return false;
}

function isErrorContinuation(line: string, prevNonEmpty: string | null): boolean {
  const normalizedLine = normalizeForDetection(line);
  if (normalizedLine.trim() === "") return false;
  if (/^\s{2,}\S/.test(normalizedLine)) return true;
  const t = normalizedLine.trimStart();
  if (/^at\s/.test(t)) return true;
  if (/^at\s+[\w$.]+\s*\(/.test(t)) return true;
  const lower = t.toLowerCase();
  if (lower.startsWith("caused by:")) return true;
  if (lower.startsWith("warning:") && prevNonEmpty && isErrorStarter(prevNonEmpty)) return true;
  if (/^[a-z]+:\s+/.test(t) && prevNonEmpty && isErrorStarter(prevNonEmpty)) return true;
  if (/^\s*\|\s*$/.test(normalizedLine)) return true;
  if (/^\s*\^\s*$/.test(normalizedLine.trim())) return true;
  return false;
}

export function segmentConsoleLines(lines: string[]): ConsoleSegment[] {
  if (lines.length === 0) return [];

  const segments: ConsoleSegment[] = [];
  let i = 0;

  while (i < lines.length) {
    const line = lines[i];

    if (isExplicitErrorBlockStart(line)) {
      const errStart = i;
      i++;
      while (i < lines.length && !isExplicitErrorBlockEnd(lines[i])) {
        i++;
      }
      if (i < lines.length) i++;
      const chunk = trimTrailingEmpty(lines.slice(errStart, i));
      if (chunk.length > 0) {
        segments.push({ kind: "error", text: chunk.join("\n") });
      }
      continue;
    }

    if (!isErrorStarter(line)) {
      const plainStart = i;
      while (
        i < lines.length &&
        !isErrorStarter(lines[i]) &&
        !isExplicitErrorBlockStart(lines[i])
      ) {
        i++;
      }
      const chunk = trimTrailingEmpty(lines.slice(plainStart, i));
      if (chunk.length > 0) {
        segments.push({ kind: "plain", text: chunk.join("\n") });
      }
      continue;
    }

    const errStart = i;
    i++;
    let prevNonEmpty = lines[errStart];
    while (i < lines.length) {
      const cur = lines[i];
      if (cur.trim() === "") break;
      if (!isErrorContinuation(cur, prevNonEmpty)) break;
      prevNonEmpty = cur.trim() ? cur : prevNonEmpty;
      i++;
    }
    const chunk = trimTrailingEmpty(lines.slice(errStart, i));
    if (chunk.length > 0) {
      segments.push({ kind: "error", text: chunk.join("\n") });
    }
  }

  return segments;
}

