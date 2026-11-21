import { db } from './index';

/**
 * Agent = persona / expert (แบบ grok bot): ชื่อ, คำอธิบาย, system prompt, โมเดล
 * ไม่ใช่ process ที่รันงานเบื้องหลัง — แชทกับ agent จะใช้ persona นี้ตอน generate
 */
export interface AgentRow {
	id: string;
	name: string;
	description: string;
	system_prompt: string;
	model_id: string | null;
	avatar: string;
	created_at: number;
	updated_at: number;
}

export interface AgentInput {
	name: string;
	description?: string;
	systemPrompt?: string;
	modelId?: string | null;
	avatar?: string;
}

const SELECT =
	'SELECT id, name, description, system_prompt, model_id, avatar, created_at, updated_at FROM agents';

export function listAgents(): AgentRow[] {
	return db.query<AgentRow, []>(`${SELECT} ORDER BY updated_at DESC`).all();
}

export function getAgent(id: string): AgentRow | null {
	return db.query<AgentRow, [string]>(`${SELECT} WHERE id = ?`).get(id);
}

export function createAgent(input: AgentInput): AgentRow {
	const now = Date.now();
	const id = crypto.randomUUID();
	const name = input.name.trim().slice(0, 120) || 'Untitled agent';

	db.query(
		`INSERT INTO agents (id, name, description, system_prompt, model_id, avatar, created_at, updated_at)
		 VALUES (?, ?, ?, ?, ?, ?, ?, ?)`
	).run(
		id,
		name,
		input.description?.trim() ?? '',
		input.systemPrompt?.trim() ?? '',
		input.modelId?.trim() || null,
		input.avatar?.trim() ?? '',
		now,
		now
	);

	return getAgent(id)!;
}

export function updateAgent(id: string, input: Partial<AgentInput>): AgentRow | null {
	const current = getAgent(id);
	if (!current) return null;

	const name = input.name !== undefined ? input.name.trim().slice(0, 120) || current.name : current.name;

	db.query(
		`UPDATE agents
		 SET name = ?, description = ?, system_prompt = ?, model_id = ?, avatar = ?, updated_at = ?
		 WHERE id = ?`
	).run(
		name,
		input.description ?? current.description,
		input.systemPrompt ?? current.system_prompt,
		input.modelId !== undefined ? input.modelId?.trim() || null : current.model_id,
		input.avatar ?? current.avatar,
		Date.now(),
		id
	);

	return getAgent(id);
}

export function deleteAgent(id: string) {
	db.query('DELETE FROM agents WHERE id = ?').run(id);
}

export function agentChatCount(agentId: string): number {
	return db.query<{ n: number }, [string]>('SELECT COUNT(*) AS n FROM chats WHERE agent_id = ?').get(agentId)?.n ?? 0;
}
