/** จัดรูปแบบเวลาเป็น string ฝั่ง server เท่านั้น — กัน hydration mismatch ระหว่าง TZ ของ server กับ browser */
export function when(ms: number | null | undefined): string {
	if (!ms) return '';
	return new Date(ms).toLocaleString('th-TH', {
		day: 'numeric',
		month: 'short',
		hour: '2-digit',
		minute: '2-digit'
	});
}
