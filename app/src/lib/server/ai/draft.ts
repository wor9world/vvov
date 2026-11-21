import { generateText, Output } from 'ai';
import { z } from 'zod';
import { draftModelCandidates, languageModel } from './models';

/**
 * ให้ AI ร่างเนื้อหาสำหรับสร้าง agent/project
 * ลองโมเดลตามลำดับ (gemini → deepseek → groq → ollama) จนกว่าจะสำเร็จ
 * เพื่อให้ยังใช้งานได้แม้ provider ตัวใดตัวหนึ่ง key หมดอายุ/ล่ม
 */

const agentSchema = z.object({
	name: z.string().describe('ชื่อ agent สั้น ๆ ไม่เกิน 40 ตัวอักษร'),
	description: z.string().describe('คำอธิบาย 1-2 ประโยค ว่าถนัดอะไร'),
	systemPrompt: z
		.string()
		.describe('system prompt ที่ระบุบทบาท ความเชี่ยวชาญ กติกาการตอบ และน้ำเสียง'),
	avatar: z.string().describe('emoji 1 ตัวที่เข้ากับ persona')
});

const projectSchema = z.object({
	name: z.string().describe('ชื่อ project สั้น ๆ ไม่เกิน 40 ตัวอักษร'),
	description: z.string().describe('คำอธิบาย 1-2 ประโยค'),
	instructions: z
		.string()
		.describe('instructions ที่จะถูกส่งให้โมเดลทุกแชทใน project: บริบท เป้าหมาย กติกา')
});

export type AgentDraft = z.infer<typeof agentSchema>;
export type ProjectDraft = z.infer<typeof projectSchema>;

const SYSTEM = `คุณเป็นผู้ช่วยออกแบบ persona/agent และ project สำหรับ AI workspace
ตอบเป็นภาษาเดียวกับคำอธิบายของผู้ใช้ (ถ้าเป็นไทยให้ตอบไทย)
เขียนให้กระชับ ใช้งานได้จริง ไม่ใส่ marketing fluff`;

/** ลองสร้างด้วยโมเดลทีละตัวจนสำเร็จ */
async function withFallback<T>(run: (modelId: string) => Promise<T>): Promise<T> {
	let lastError: unknown = new Error('ไม่มีโมเดลสำหรับร่างข้อความ');

	for (const modelId of draftModelCandidates()) {
		try {
			return await run(modelId);
		} catch (cause) {
			lastError = cause;
			console.warn(`[draft] ${modelId} ใช้ไม่ได้:`, String(cause).slice(0, 200));
		}
	}

	throw lastError;
}

export async function draftAgent(prompt: string): Promise<AgentDraft> {
	return withFallback(async (modelId) => {
		const { output } = await generateText({
			model: languageModel(modelId),
			system: SYSTEM,
			output: Output.object({ schema: agentSchema }),
			prompt: `สร้าง agent persona จากคำอธิบายนี้:\n"""${prompt}"""\n\n
- name: สั้น จำง่าย
- description: 1-2 ประโยค
- systemPrompt: บทบาท + ความเชี่ยวชาญ + กติกาการตอบ + น้ำเสียง (ไม่ต้องใส่ชื่อผู้ใช้)
- avatar: emoji 1 ตัว
- ห้ามแต่งข้อมูลที่ไม่เกี่ยวกับคำอธิบาย`
		});

		return output;
	});
}

export async function draftProject(prompt: string): Promise<ProjectDraft> {
	return withFallback(async (modelId) => {
		const { output } = await generateText({
			model: languageModel(modelId),
			system: SYSTEM,
			output: Output.object({ schema: projectSchema }),
			prompt: `สร้าง project สำหรับ AI workspace จากคำอธิบายนี้:\n"""${prompt}"""\n\n
- name: สั้น จำง่าย
- description: 1-2 ประโยค
- instructions: บริบทของ project + สิ่งที่โมเดลต้องยึดถือทุกแชท + ขอบเขตงาน
- ห้ามแต่งข้อมูลที่ไม่เกี่ยวกับคำอธิบาย`
		});

		return output;
	});
}
