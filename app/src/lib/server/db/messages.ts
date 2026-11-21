import type { UIMessage } from 'ai';
import { db } from './index';
import { maybeTitleFromFirstMessage, touchChat } from './chats';

export type MessageRole = 'system' | 'user' | 'assistant';
export type MessageStatus = 'streaming' | 'complete' | 'error' | 'interrupted';

export interface MessageRow {
	id: string;
	chat_id: string;
	parent_id: string | null;
	seq: number;
	role: MessageRole;
	parts: string;
	text: string;
	status: MessageStatus;
	model_id: string | null;
	usage: string | null;
	error: string | null;
	created_at: number;
}

export interface SaveOptions {
	status?: MessageStatus;
	modelId?: string | null;
	error?: string | null;
}

const INSERT = `
	INSERT INTO messages
		(id, chat_id, parent_id, seq, role, parts, text, status, model_id, usage, error, created_at)
	VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
	ON CONFLICT(id) DO UPDATE SET
		parts  = excluded.parts,
		text   = excluded.text,
		status = excluded.status
`;

/** ข้อความล้วนจาก parts — ใช้ทำ title/preview/search โดยไม่ต้อง parse JSON ทุกครั้ง */
export function textOf(parts: UIMessage['parts']): string {
	const out: string[] = [];
	for (const part of parts) {
		if (part.type === 'text' && part.text) out.push(part.text);
	}
	return out.join('\n\n');
}

function toUIMessage(row: MessageRow): UIMessage {
	return {
		id: row.id,
		role: row.role,
		parts: JSON.parse(row.parts) as UIMessage['parts']
	};
}

/** เอา N ข้อความล่าสุด (เรียงจากเก่า → ใหม่) */
export function listMessages(chatId: string, limit = 200): UIMessage[] {
	const rows = db
		.query<MessageRow, [string, number]>(
			`SELECT * FROM (SELECT * FROM messages WHERE chat_id = ? ORDER BY seq DESC LIMIT ?) ORDER BY seq ASC`
		)
		.all(chatId, limit);

	return rows.map(toUIMessage);
}

export function countMessages(chatId: string): number {
	return db.query<{ n: number }, [string]>('SELECT COUNT(*) AS n FROM messages WHERE chat_id = ?').get(chatId)?.n ?? 0;
}

export function getMessageSeq(chatId: string, messageId: string): number | null {
	return (
		db
			.query<{ seq: number }, [string, string]>('SELECT seq FROM messages WHERE chat_id = ? AND id = ?')
			.get(chatId, messageId)?.seq ?? null
	);
}

/** ลบข้อความตั้งแต่ seq ที่กำหนด (ใช้ตอน regenerate) */
export function deleteMessagesFrom(chatId: string, seq: number) {
	db.query('DELETE FROM messages WHERE chat_id = ? AND seq >= ?').run(chatId, seq);
}

/**
 * บันทึก UIMessage[] ลง DB แบบ idempotent
 * - id มีอยู่แล้ว → update parts/text/status
 * - ใหม่ → ต่อ seq ท้ายสุด
 */
export function saveMessages(chatId: string, uiMessages: UIMessage[], opts: SaveOptions = {}) {
	const now = Date.now();
	const status: MessageStatus = opts.status ?? 'complete';

	const seqById = new Map<string, number>();
	for (const row of db.query<{ id: string; seq: number }, [string]>('SELECT id, seq FROM messages WHERE chat_id = ?').all(chatId)) {
		seqById.set(row.id, row.seq);
	}
	let nextSeq = 0;
	for (const seq of seqById.values()) nextSeq = Math.max(nextSeq, seq);

	const insert = db.query(INSERT);

	db.transaction(() => {
		for (const message of uiMessages) {
			const text = textOf(message.parts);
			const seq = seqById.get(message.id) ?? ++nextSeq;

			insert.run(
				message.id,
				chatId,
				null,
				seq,
				message.role,
				JSON.stringify(message.parts),
				text,
				status,
				message.role === 'assistant' ? (opts.modelId ?? null) : null,
				null,
				opts.error ?? null,
				now
			);
		}

		touchChat(chatId, now);

		const firstUser = uiMessages.find((m) => m.role === 'user' && textOf(m.parts));
		if (firstUser) maybeTitleFromFirstMessage(chatId, textOf(firstUser.parts));
	})();
}
