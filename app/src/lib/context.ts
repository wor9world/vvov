export type ChatGroupKey = 'today' | 'yesterday' | 'week' | 'older';

/** จัดกลุ่มแชทตามวันที่ — ใช้ร่วมกันทั้ง server และ client, ไม่แตะ DB */
export function chatGroup(ts: number, now = Date.now()): ChatGroupKey {
	const start = new Date(now);
	start.setHours(0, 0, 0, 0);
	const today = start.getTime();

	if (ts >= today) return 'today';
	if (ts >= today - 86_400_000) return 'yesterday';
	if (ts >= today - 7 * 86_400_000) return 'week';
	return 'older';
}

export interface WorkspaceCounts {
	chats: number;
	agents: number;
	projects: number;
}

/** สรุปบริบทของ route ปัจจุบัน สำหรับ panel ด้านขวา */
export type ContextSummary =
	| {
			kind: 'chat';
			id: string;
			title: string;
			modelId: string | null;
			agentName: string | null;
			projectName: string | null;
			messageCount: number;
		}
	| {
			kind: 'agent';
			id: string;
			name: string;
			description: string;
			modelId: string | null;
			chatCount: number;
		}
	| {
			kind: 'project';
			id: string;
			name: string;
			description: string;
			chatCount: number;
		}
	| { kind: 'workspace'; counts: WorkspaceCounts };
