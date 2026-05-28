export type ShardInfo = { index: number; total: number };

const SHARD_RE = /^(.*)-(\d{5})-of-(\d{5})\.gguf$/i;

export function shardInfo(file: string): ShardInfo | null {
  const base = file.split("/").pop() ?? file;
  const m = SHARD_RE.exec(base);
  if (!m) return null;
  return { index: parseInt(m[2], 10), total: parseInt(m[3], 10) };
}

export function shardKey(file: string): string {
  const m = SHARD_RE.exec(file);
  if (!m) return file;
  return `${m[1]}.gguf`;
}

export function shardBaseName(file: string): string {
  const base = file.split("/").pop() ?? file;
  return shardKey(base);
}

export type Group<T> = {
  key: string;
  rep: T;
  members: T[];
  totalSize: number;
  shardCount: number;
};

export function groupShards<T extends { path?: string; file: string; size: number }>(
  items: T[],
): Group<T>[] {
  const map = new Map<string, T[]>();
  for (const it of items) {
    const key = shardKey(it.file);
    const arr = map.get(key) ?? [];
    arr.push(it);
    map.set(key, arr);
  }
  const groups: Group<T>[] = [];
  for (const [key, members] of map) {
    members.sort((a, b) => {
      const ai = shardInfo(a.file)?.index ?? 0;
      const bi = shardInfo(b.file)?.index ?? 0;
      return ai - bi;
    });
    groups.push({
      key,
      rep: members[0],
      members,
      totalSize: members.reduce((s, m) => s + m.size, 0),
      shardCount: members.length,
    });
  }
  return groups;
}
