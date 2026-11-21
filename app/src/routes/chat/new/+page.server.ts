import type { PageServerLoad } from './$types';
import { listModels } from '#lib/server/ai/models';

// ยังไม่สร้างแถวใน DB — แค่ให้ id ที่จะใช้ตอนส่งข้อความแรก + รายการโมเดลให้เลือก
export const load: PageServerLoad = async () => {
	return {
		chatId: crypto.randomUUID(),
		models: await listModels()
	};
};
