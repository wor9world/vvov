import { db } from './index';

/**
 * Project = กลุ่มแชท + instructions ที่ inject เข้า system prompt ของทุกแชทใน project
 * (ยังไม่มีไฟล์/โฟลเดอร์ทำงาน — รอ §9 ข้อ 2)
 */
export interface ProjectRow {
	id: string;
	name: string;
	description: string;
	instructions: string;
	created_at: number;
	updated_at: number;
}

export interface ProjectInput {
	name: string;
	description?: string;
	instructions?: string;
}

const SELECT = 'SELECT id, name, description, instructions, created_at, updated_at FROM projects';

export function listProjects(): ProjectRow[] {
	return db.query<ProjectRow, []>(`${SELECT} ORDER BY updated_at DESC`).all();
}

export function getProject(id: string): ProjectRow | null {
	return db.query<ProjectRow, [string]>(`${SELECT} WHERE id = ?`).get(id);
}

export function createProject(input: ProjectInput): ProjectRow {
	const now = Date.now();
	const id = crypto.randomUUID();
	const name = input.name.trim().slice(0, 120) || 'Untitled project';

	db.query(
		`INSERT INTO projects (id, name, description, instructions, created_at, updated_at)
		 VALUES (?, ?, ?, ?, ?, ?)`
	).run(id, name, input.description?.trim() ?? '', input.instructions?.trim() ?? '', now, now);

	return getProject(id)!;
}

export function updateProject(id: string, input: Partial<ProjectInput>): ProjectRow | null {
	const current = getProject(id);
	if (!current) return null;

	const name = input.name !== undefined ? input.name.trim().slice(0, 120) || current.name : current.name;

	db.query(
		`UPDATE projects SET name = ?, description = ?, instructions = ?, updated_at = ? WHERE id = ?`
	).run(
		name,
		input.description ?? current.description,
		input.instructions ?? current.instructions,
		Date.now(),
		id
	);

	return getProject(id);
}

export function deleteProject(id: string) {
	db.query('DELETE FROM projects WHERE id = ?').run(id);
}

export function projectChatCount(projectId: string): number {
	return (
		db.query<{ n: number }, [string]>('SELECT COUNT(*) AS n FROM chats WHERE project_id = ?').get(projectId)?.n ?? 0
	);
}
