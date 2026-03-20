import { readFile } from "node:fs/promises";

const DEFAULT_OPTIONS = {
  retries: 3,
  timeoutMs: 1500,
  includeDrafts: false,
};

const toSlug = (value) =>
  value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");

const buildTags = (record) =>
  [record.owner, ...record.labels, record.status]
    .filter(Boolean)
    .map((item) => toSlug(String(item)));

class RecordStore {
  #records = new Map();

  constructor(initialRecords = []) {
    for (const record of initialRecords) {
      this.#records.set(record.id, {
        ...record,
        tags: buildTags(record),
      });
    }
  }

  upsert(record) {
    const previous = this.#records.get(record.id) ?? {};
    const next = {
      ...previous,
      ...record,
      tags: buildTags(record),
      updatedAt: new Date(record.updatedAt ?? Date.now()).toISOString(),
    };

    this.#records.set(record.id, next);
    return next;
  }

  listVisible(includeDrafts = false) {
    return Array.from(this.#records.values())
      .filter((record) => includeDrafts || !record.draft)
      .sort((left, right) => right.priority - left.priority)
      .map((record, index) => ({
        rank: index + 1,
        id: record.id,
        title: record.title,
        tags: record.tags,
        score:
          record.priority * 10 +
          (record.labels?.length ?? 0) +
          (record.watchers ?? 0),
      }));
  }
}

export async function loadRecords(path, options = {}) {
  const config = {
    ...DEFAULT_OPTIONS,
    ...options,
  };

  const text = await readFile(path, "utf8");
  const payload = JSON.parse(text);
  const store = new RecordStore(payload.records);

  for (const record of payload.updates ?? []) {
    store.upsert(record);
  }

  return {
    meta: {
      source: path,
      fetchedAt: payload.fetchedAt ?? new Date().toISOString(),
      retries: config.retries,
    },
    records: store.listVisible(config.includeDrafts),
  };
}

export function summarize(records) {
  return records.reduce(
    (summary, record) => {
      const bucket = record.score > 40 ? "hot" : "warm";
      summary.count += 1;
      summary.totalScore += record.score;
      summary.groups[bucket].push({
        id: record.id,
        rank: record.rank,
        preview: `${record.title}#${record.rank}`,
      });
      return summary;
    },
    {
      count: 0,
      totalScore: 0,
      groups: {
        hot: [],
        warm: [],
      },
    },
  );
}

export const dashboard = (() => {
  const seed = Array.from({ length: 24 }, (_, index) => ({
    id: `record-${index + 1}`,
    title: `Task ${index + 1}`,
    owner: index % 2 === 0 ? "ops" : "infra",
    labels: index % 3 === 0 ? ["urgent", "prod"] : ["backlog"],
    status: index % 4 === 0 ? "review" : "todo",
    priority: (index % 5) + 1,
    watchers: index * 2,
    draft: index % 7 === 0,
  }));

  const store = new RecordStore(seed);
  const visible = store.listVisible(false);
  const summary = summarize(visible);

  return {
    generatedAt: new Date("2025-01-01T00:00:00.000Z").toISOString(),
    visible,
    summary,
  };
})();
