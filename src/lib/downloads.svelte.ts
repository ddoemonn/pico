import {
  cancelDownload as cancelCmd,
  downloadModel,
  onDownloadProgress,
  type DownloadProgress,
} from "./api";

export type DlEntry = {
  repo: string;
  file: string;
  total: number;
  downloaded: number;
  groupKey: string;
  startedAt: number;
};

function key(repo: string, file: string): string {
  return `${repo}/${file}`;
}

class DownloadsStore {
  items = $state<Record<string, DlEntry>>({});
  completedAt = $state(0);

  init() {
    onDownloadProgress((p: DownloadProgress) => {
      const k = key(p.repo, p.file);
      const existing = this.items[k];
      if (existing) {
        existing.downloaded = p.downloaded;
        if (p.total) existing.total = p.total;
      }
    });
  }

  list(): DlEntry[] {
    return Object.values(this.items).sort((a, b) => a.startedAt - b.startedAt);
  }

  totals(): { done: number; total: number; count: number } {
    let done = 0;
    let total = 0;
    let count = 0;
    for (const e of Object.values(this.items)) {
      done += e.downloaded;
      total += e.total;
      count += 1;
    }
    return { done, total, count };
  }

  async pull(repo: string, file: string, total: number, groupKey: string) {
    const k = key(repo, file);
    this.items[k] = {
      repo,
      file,
      total,
      downloaded: 0,
      groupKey,
      startedAt: Date.now(),
    };
    try {
      await downloadModel(repo, file);
      this.completedAt = Date.now();
    } finally {
      delete this.items[k];
      this.items = { ...this.items };
    }
  }

  async cancel(repo: string, file: string) {
    await cancelCmd(repo, file).catch(() => {});
  }

  async cancelGroup(repo: string, groupKey: string) {
    const matches = Object.values(this.items).filter(
      (e) => e.repo === repo && e.groupKey === groupKey,
    );
    for (const m of matches) await this.cancel(m.repo, m.file);
  }

  hasActive(repo: string, groupKey: string): boolean {
    return Object.values(this.items).some(
      (e) => e.repo === repo && e.groupKey === groupKey,
    );
  }

  groupProgress(repo: string, groupKey: string): { done: number; total: number } {
    let done = 0;
    let total = 0;
    for (const e of Object.values(this.items)) {
      if (e.repo === repo && e.groupKey === groupKey) {
        done += e.downloaded;
        total += e.total;
      }
    }
    return { done, total };
  }
}

export const downloads = new DownloadsStore();
