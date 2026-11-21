export interface ToolPartLike {
	type: string;
	toolName?: string;
	toolCallId?: string;
	state?: string;
	input?: unknown;
	output?: unknown;
	errorText?: string;
}

/** AI SDK 7 ใช้ `tool-<name>` สำหรับ tool ที่ประกาศไว้ และ `dynamic-tool` สำหรับ tool ที่ไม่รู้จัก */
export function isToolPart(part: { type: string }): part is { type: string } & ToolPartLike {
	return part.type === 'dynamic-tool' || part.type.startsWith('tool-');
}
