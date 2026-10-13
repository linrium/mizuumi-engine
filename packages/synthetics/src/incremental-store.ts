import { mkdirSync } from "node:fs"
import { dirname } from "node:path"
import Database from "better-sqlite3"
import { and, eq, inArray, isNull } from "drizzle-orm"
import { drizzle } from "drizzle-orm/better-sqlite3"
import { integer, sqliteTable, text } from "drizzle-orm/sqlite-core"
import type { IncrementalEvent } from "./events.js"
import type { IncrementalStreamState } from "./incremental.js"

const streams = sqliteTable("synthetic_streams", {
  id: text("id").primaryKey(),
  region: text("region").notNull(),
  seed: integer("seed").notNull(),
  cursor: text("cursor").notNull(),
  tick: integer("tick").notNull(),
  stateJson: text("state_json").notNull(),
  createdAt: text("created_at").notNull(),
  updatedAt: text("updated_at").notNull(),
})

const outbox = sqliteTable("synthetic_outbox", {
  id: text("id").primaryKey(),
  streamId: text("stream_id").notNull(),
  source: text("source").notNull(),
  eventKey: text("event_key").notNull(),
  payloadJson: text("payload_json").notNull(),
  createdAt: text("created_at").notNull(),
  publishedAt: text("published_at"),
})

export class StreamAlreadyExistsError extends Error {}
export class StreamNotFoundError extends Error {}
export class StreamConflictError extends Error {}

export class IncrementalStore {
  private readonly sqlite: Database.Database
  private readonly orm

  constructor(path: string) {
    if (path !== ":memory:") mkdirSync(dirname(path), { recursive: true })
    this.sqlite = new Database(path)
    this.sqlite.pragma("journal_mode = WAL")
    this.sqlite.pragma("foreign_keys = ON")
    this.sqlite.exec(`
      CREATE TABLE IF NOT EXISTS synthetic_streams (
        id TEXT PRIMARY KEY,
        region TEXT NOT NULL,
        seed INTEGER NOT NULL,
        cursor TEXT NOT NULL,
        tick INTEGER NOT NULL,
        state_json TEXT NOT NULL,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
      );
      CREATE TABLE IF NOT EXISTS synthetic_outbox (
        id TEXT PRIMARY KEY,
        stream_id TEXT NOT NULL,
        source TEXT NOT NULL,
        event_key TEXT NOT NULL,
        payload_json TEXT NOT NULL,
        created_at TEXT NOT NULL,
        published_at TEXT,
        FOREIGN KEY(stream_id) REFERENCES synthetic_streams(id) ON DELETE CASCADE
      );
      CREATE INDEX IF NOT EXISTS synthetic_outbox_pending_idx
        ON synthetic_outbox(stream_id, published_at, created_at);
    `)
    this.orm = drizzle(this.sqlite)
  }

  getStream(streamId: string): IncrementalStreamState | undefined {
    const row = this.orm.select().from(streams).where(eq(streams.id, streamId)).get()
    return row ? (JSON.parse(row.stateJson) as IncrementalStreamState) : undefined
  }

  createStream(state: IncrementalStreamState, events: IncrementalEvent[]) {
    const transaction = this.sqlite.transaction(() => {
      const existing = this.orm.select({ id: streams.id }).from(streams).where(eq(streams.id, state.streamId)).get()
      if (existing) throw new StreamAlreadyExistsError(`Stream already exists: ${state.streamId}`)
      this.orm
        .insert(streams)
        .values({
          id: state.streamId,
          region: state.region,
          seed: state.seed,
          cursor: state.cursor,
          tick: state.tick,
          stateJson: JSON.stringify(state),
          createdAt: state.createdAt,
          updatedAt: state.cursor,
        })
        .run()
      this.insertOutbox(events)
    })
    transaction()
  }

  advanceStream(
    previousCursor: string,
    state: IncrementalStreamState,
    events: IncrementalEvent[],
  ) {
    const transaction = this.sqlite.transaction(() => {
      const result = this.orm
        .update(streams)
        .set({
          cursor: state.cursor,
          tick: state.tick,
          stateJson: JSON.stringify(state),
          updatedAt: state.cursor,
        })
        .where(and(eq(streams.id, state.streamId), eq(streams.cursor, previousCursor)))
        .run()
      if (result.changes !== 1) {
        throw new StreamConflictError(`Stream was advanced concurrently: ${state.streamId}`)
      }
      this.insertOutbox(events)
    })
    transaction()
  }

  private insertOutbox(events: IncrementalEvent[]) {
    if (events.length === 0) return
    this.orm
      .insert(outbox)
      .values(
        events.map((event) => ({
          id: event.id,
          streamId: event.streamId,
          source: event.source,
          eventKey: event.subject,
          payloadJson: JSON.stringify(event),
          createdAt: event.time,
        })),
      )
      .run()
  }

  pendingEvents(streamId: string): IncrementalEvent[] {
    return this.orm
      .select({ payloadJson: outbox.payloadJson })
      .from(outbox)
      .where(and(eq(outbox.streamId, streamId), isNull(outbox.publishedAt)))
      .orderBy(outbox.createdAt)
      .all()
      .map((row) => JSON.parse(row.payloadJson) as IncrementalEvent)
  }

  markPublished(eventIds: string[], publishedAt: string) {
    if (eventIds.length === 0) return
    this.orm
      .update(outbox)
      .set({ publishedAt })
      .where(inArray(outbox.id, eventIds))
      .run()
  }

  close() {
    this.sqlite.close()
  }
}
