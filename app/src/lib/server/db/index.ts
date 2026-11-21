import { Database } from 'bun:sqlite';
import { mkdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';

/**
 * SQLite ผ่าน bun:sqlite ตรง ๆ — ไม่มี ORM
 *
 * - process เดียวเป็นเจ้าของไฟล์ (SvelteKit/Bun) ห้าม process อื่นเปิด
 * - WAL + busy_timeout: อ่านระหว่างเขียนได้
 * - migration ใช้ PRAGMA user_version + DDL ที่อยู่ในไฟล์นี้ (ไม่ต้อง copy ไฟล์ .sql ตอน build)
 *
 * เปลี่ยน DATABASE_PATH ได้ผ่าน env; default = ./data/app.db (relative to cwd)
 */

export const DB_PATH = resolve(process.env.DATABASE_PATH ?? 'data/app.db');

const MIGRATIONS: { version: number; name: string; sql: string }[] = [
	{
		version: 1,
		name: 'init',
		sql: `
			CREATE TABLE chats (
				id              TEXT PRIMARY KEY,
				title           TEXT NOT NULL DEFAULT 'New chat',
				model_id        TEXT,
				metadata        TEXT NOT NULL DEFAULT '{}',
				created_at      INTEGER NOT NULL,
				updated_at      INTEGER NOT NULL,
				last_message_at INTEGER
			);
			CREATE INDEX chats_updated ON chats(updated_at DESC);

			CREATE TABLE messages (
				id         TEXT PRIMARY KEY,
				chat_id    TEXT NOT NULL REFERENCES chats(id) ON DELETE CASCADE,
				parent_id  TEXT,
				seq        INTEGER NOT NULL,
				role       TEXT NOT NULL CHECK (role IN ('user','assistant','system')),
				parts      TEXT NOT NULL,
				text       TEXT NOT NULL DEFAULT '',
				status     TEXT NOT NULL DEFAULT 'complete'
				           CHECK (status IN ('streaming','complete','error','interrupted')),
				model_id   TEXT,
				usage      TEXT,
				error      TEXT,
				created_at INTEGER NOT NULL
			);
			CREATE UNIQUE INDEX messages_chat_seq ON messages(chat_id, seq);
			CREATE INDEX messages_chat_created ON messages(chat_id, created_at);

			CREATE TABLE settings (
				key        TEXT PRIMARY KEY,
				value      TEXT NOT NULL,
				updated_at INTEGER NOT NULL
			);
		`
	},
	{
		version: 2,
		name: 'agents-projects',
		sql: `
			CREATE TABLE agents (
				id            TEXT PRIMARY KEY,
				name          TEXT NOT NULL,
				description   TEXT NOT NULL DEFAULT '',
				system_prompt TEXT NOT NULL DEFAULT '',
				model_id      TEXT,
				avatar        TEXT NOT NULL DEFAULT '',
				created_at    INTEGER NOT NULL,
				updated_at    INTEGER NOT NULL
			);
			CREATE INDEX agents_name ON agents(name);

			CREATE TABLE projects (
				id           TEXT PRIMARY KEY,
				name         TEXT NOT NULL,
				description  TEXT NOT NULL DEFAULT '',
				instructions TEXT NOT NULL DEFAULT '',
				created_at   INTEGER NOT NULL,
				updated_at   INTEGER NOT NULL
			);
			CREATE INDEX projects_name ON projects(name);

			ALTER TABLE chats ADD COLUMN agent_id   TEXT REFERENCES agents(id)   ON DELETE SET NULL;
			ALTER TABLE chats ADD COLUMN project_id TEXT REFERENCES projects(id) ON DELETE SET NULL;
			CREATE INDEX chats_agent   ON chats(agent_id, updated_at DESC);
			CREATE INDEX chats_project ON chats(project_id, updated_at DESC);
		`
	}
];

function openDatabase(): Database {
	mkdirSync(dirname(DB_PATH), { recursive: true });

	const sqlite = new Database(DB_PATH, { create: true });

	// ต้องอยู่นอก transaction และตั้งทุก connection
	sqlite.exec(`
		PRAGMA journal_mode = WAL;
		PRAGMA synchronous = NORMAL;
		PRAGMA foreign_keys = ON;
		PRAGMA busy_timeout = 5000;
	`);

	migrate(sqlite);
	return sqlite;
}

function migrate(sqlite: Database) {
	const row = sqlite.query<{ user_version: number }, []>('PRAGMA user_version').get();
	const current = row?.user_version ?? 0;

	for (const migration of MIGRATIONS) {
		if (migration.version <= current) continue;

		sqlite.transaction(() => {
			sqlite.exec(migration.sql);
			sqlite.exec(`PRAGMA user_version = ${migration.version}`);
		})();

		console.info(`[db] migrated → v${migration.version} (${migration.name})`);
	}
}

// singleton — กัน dev/HMR เปิด connection ซ้ำ
const globalForDb = globalThis as unknown as { __vvovDb?: Database };

export const db: Database = (globalForDb.__vvovDb ??= openDatabase());

export function schemaVersion(): number {
	return db.query<{ user_version: number }, []>('PRAGMA user_version').get()?.user_version ?? 0;
}
