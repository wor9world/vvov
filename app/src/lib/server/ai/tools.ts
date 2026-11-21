import { tool } from 'ai';
import { z } from 'zod';

/**
 * Tools ที่โมเดลเรียกได้ระหว่างแชท
 * - ทุก tool ต้องคืนค่า JSON-serializable (จะถูกเก็บลง messages.parts)
 * - ห้าม throw ออกมา ให้คืน { ok: false, error } เพื่อให้โมเดลอธิบายต่อได้
 */

const ENGINE_URL = process.env.WOR_ENGINE_URL ?? 'http://127.0.0.1:8080';

export const mathTool = tool({
	description:
		'คำนวณหรือแก้โจทย์คณิตศาสตร์ด้วย engine ภายในเครื่อง (เลขคณิต, สมการ, ระบบสมการ, อนุพันธ์, อินทิกรัล, Boolean SAT). ใช้เมื่อต้องได้ผลที่แน่นอน แทนการเดา',
	inputSchema: z.object({
		query: z
			.string()
			.describe('โจทย์ เช่น "23x+100=491", "x^2+y^2=25, x-y=1", "diff: 4x^3-5x^2+2x-9", "sat: (A & B) -> A"')
	}),
	execute: async ({ query }) => {
		try {
			const response = await fetch(`${ENGINE_URL}/api/v1/execute`, {
				method: 'POST',
				headers: { 'content-type': 'application/json' },
				body: JSON.stringify({ query }),
				signal: AbortSignal.timeout(8000)
			});

			if (!response.ok) {
				return { ok: false, error: `engine HTTP ${response.status}` };
			}

			const data = (await response.json()) as {
				success: boolean;
				intent: string;
				output: string;
			};

			return { ok: data.success, intent: data.intent, output: data.output };
		} catch (cause) {
			return {
				ok: false,
				error: `engine ไม่พร้อมใช้งาน (${ENGINE_URL}): ${String(cause)}`
			};
		}
	}
});

export const clockTool = tool({
	description: 'บอกวันและเวลาปัจจุบันตาม timezone ที่ระบุ',
	inputSchema: z.object({
		timezone: z.string().optional().describe('IANA timezone เช่น Asia/Bangkok (ค่าเริ่มต้น)')
	}),
	execute: ({ timezone }) => {
		const tz = timezone?.trim() || 'Asia/Bangkok';
		const now = new Date();

		return {
			timezone: tz,
			iso: now.toISOString(),
			local: now.toLocaleString('en-GB', {
				timeZone: tz,
				dateStyle: 'full',
				timeStyle: 'long'
			})
		};
	}
});

export const tools = {
	math: mathTool,
	clock: clockTool
};
