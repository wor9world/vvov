import { db } from './index';

export interface ChatRow {
	id: string;
	title: string;
	model_id: string | null;
	metadata: string;
	created_at: number;
	updated_at: number;
	last_message_at: number | null;
	agent_id: string | null;
	project_id: string | null;
}

export interface ChatListItem extends ChatRow {
	agent_name: string | null;
	project_name: string | null;
}

export interface CreateChatInput {
	id?: string;
	title?: string;
	modelId?: string | null;
	agentId?: string | null;
	projectId?: string | null;
}

const SELECT = `SELECT id, title, model_id, metadata, created_at, updated_at, last_message_at, agent_id, project_id FROM chats`;

const SELECT_LIST = `
	SELECT c.id, c.title, c.model_id, c.metadata, c.created_at, c.updated_at, c.last_message_at,
	       c.agent_id, c.project_id, a.name AS agent_name, p.name AS project_name
	FROM chats c
	LEFT JOIN agents a ON a.id = c.agent_id
	LEFT JOIN projects p ON p.id = c.project_id`;

export function listChats(limit = 50): ChatListItem[] {
	return db.query<ChatListItem, [number]>(`${SELECT_LIST} ORDER BY c.updated_at DESC LIMIT ?`).all(limit);
}

export function listChatsForAgent(agentId: string, limit = 100): ChatRow[] {
	return db
		.query<ChatRow, [string, number]>(`${SELECT} WHERE agent_id = ? ORDER BY updated_at DESC LIMIT ?`)
		.all(agentId, limit);
}

export function listChatsForProject(projectId: string, limit = 100): ChatRow[] {
	return db
		.query<ChatRow, [string, number]>(`${SELECT} WHERE project_id = ? ORDER BY updated_at DESC LIMIT ?`)
		.all(projectId, limit);
}

export function getChat(id: string): ChatRow | null {
	return db.query<ChatRow, [string]>(`${SELECT} WHERE id = ?`).get(id);
}

export function createChat(input: CreateChatInput = {}): ChatRow {
	const now = Date.now();
	const id = input.id ?? crypto.randomUUID();
	const title = input.title?.trim() || 'New chat';

	db.query(
		`INSERT INTO chats (id, title, model_id, metadata, created_at, updated_at, agent_id, project_id)
		 VALUES (?, ?, ?, ?, ?, ?, ?, ?)`
	).run(id, title, input.modelId ?? null, '{}', now, now, input.agentId ?? null, input.projectId ?? null);

	return getChat(id)!;
}

export function ensureChat(id: string, input: Omit<CreateChatInput, 'id' | 'title'> = {}): ChatRow {
	const existing = getChat(id);
	if (existing) return existing;

	const now = Date.now();
	db.query(
		`INSERT OR IGNORE INTO chats (id, title, model_id, metadata, created_at, updated_at, agent_id, project_id)
		 VALUES (?, ?, ?, ?, ?, ?, ?, ?)`
	).run(id, 'New chat', input.modelId ?? null, '{}', now, now, input.agentId ?? null, input.projectId ?? null);

	return getChat(id)!;
}

export function renameChat(id: string, title: string) {
	const clean = title.trim().slice(0, 200) || 'New chat';
	db.query('UPDATE chats SET title = ?, updated_at = ? WHERE id = ?').run(clean, Date.now(), id);
}

/** เปลี่ยนโมเดลของแชท (NULL = ใช้ default ของระบบ) */
export function setChatModel(id: string, modelId: string | null) {
	db.query('UPDATE chats SET model_id = ?, updated_at = ? WHERE id = ?').run(
		modelId?.trim() || null,
		Date.now(),
		id
	);
}

/** ตั้งชื่อจากข้อความแรก — เฉพาะเมื่อยังใช้ชื่อ default อยู่ */
export function maybeTitleFromFirstMessage(id: string, text: string) {
	const chat = getChat(id);
	if (!chat || chat.title !== 'New chat') return;

	const title = text.replace(/\s+/g, ' ').trim().slice(0, 60);
	if (!title) return;

	db.query('UPDATE chats SET title = ?, updated_at = ? WHERE id = ?').run(title, Date.now(), id);
}

export function touchChat(id: string, at = Date.now()) {
	db.query('UPDATE chats SET updated_at = ?, last_message_at = ? WHERE id = ?').run(at, at, id);
}

export function deleteChat(id: string) {
	db.query('DELETE FROM chats WHERE id = ?').run(id);
}

export function countChats(): number {
	return db.query<{ n: number }, []>('SELECT COUNT(*) AS n FROM chats').get()?.n ?? 0;
}
