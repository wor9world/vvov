import type { RequestHandler } from './$types';
import {
	convertToModelMessages,
	createUIMessageStreamResponse,
	generateId,
	streamText,
	toUIMessageStream,
	validateUIMessages,
	type UIMessage
} from 'ai';
import { DEFAULT_MODEL, SYSTEM_PROMPT, languageModel } from '#lib/server/ai/models';
import { tools } from '#lib/server/ai/tools';
import { getAgent } from '#lib/server/db/agents';
import { createChat, getChat } from '#lib/server/db/chats';
import { getProject } from '#lib/server/db/projects';
import { deleteMessagesFrom, getMessageSeq, saveMessages, type MessageStatus } from '#lib/server/db/messages';

interface ChatRequestBody {
	id?: string;
	messages?: unknown;
	trigger?: 'submit-message' | 'regenerate-message';
	messageId?: string;
	/** ใช้เฉพาะแชทใหม่ที่ยังไม่มีแถวใน DB (มาจาก /chat/new) */
	model?: string;
}

/** กันค่าแปลกปลอมจาก client — รับเฉพาะรูปแบบ model id ที่ปลอดภัย */
function cleanModelId(value: unknown): string | null {
	if (typeof value !== 'string') return null;
	const id = value.trim();
	if (!id || id.length > 120 || /\s/.test(id)) return null;
	return /^[A-Za-z0-9._:\/-]+$/.test(id) ? id : null;
}

export const POST: RequestHandler = async ({ request }) => {
	let body: ChatRequestBody;
	try {
		body = await request.json();
	} catch {
		return Response.json({ error: 'invalid JSON body' }, { status: 400 });
	}

	const chatId = typeof body.id === 'string' ? body.id : '';
	if (!chatId) return Response.json({ error: 'missing chat id' }, { status: 400 });

	let uiMessages: UIMessage[];
	try {
		uiMessages = await validateUIMessages<UIMessage>({ messages: body.messages ?? [] });
	} catch (cause) {
		return Response.json({ error: 'invalid messages', cause: String(cause) }, { status: 400 });
	}
	if (uiMessages.length === 0) return Response.json({ error: 'empty messages' }, { status: 400 });

	// แชทใหม่อาจยังไม่มีแถว — สร้างตอนนี้พร้อมโมเดลที่ผู้ใช้เลือก (ถ้ามี)
	const existing = getChat(chatId);
	const chat = existing ?? createChat({ id: chatId, modelId: cleanModelId(body.model) });

	// persona/instructions ต้องมาจาก DB เท่านั้น — ห้ามรับ system prompt จาก client
	const agent = chat.agent_id ? getAgent(chat.agent_id) : null;
	const project = chat.project_id ? getProject(chat.project_id) : null;
	const system = [project?.instructions, agent?.system_prompt].filter(Boolean).join('\n\n') || SYSTEM_PROMPT;
	const modelId = agent?.model_id ?? chat.model_id ?? DEFAULT_MODEL;

	// เก็บข้อความฝั่ง user ก่อนเริ่ม generate — ถ้าโมเดลล้ม ผู้ใช้ยังไม่เสียข้อความ
	saveMessages(chatId, uiMessages, { status: 'complete' });

	const result = streamText({
		model: languageModel(modelId),
		system,
		messages: await convertToModelMessages(uiMessages),
		abortSignal: request.signal,
		temperature: 0.7,
		maxOutputTokens: 2048,
		tools,
		// อนุญาตให้โมเดลเรียก tool ได้หลายรอบ แต่ไม่เกิน 4 steps กัน loop
		stopWhen: ({ steps }) => steps.length >= 4
	});

	const stream = toUIMessageStream({
		stream: result.stream,
		originalMessages: uiMessages,
		generateMessageId: generateId,
		onError: (error) => {
			console.error('[chat] stream error', error);
			return 'Model error';
		},
		onEnd: ({ responseMessage, outcome }) => {
			const status: MessageStatus =
				outcome.status === 'failed' ? 'error' : outcome.status === 'aborted' ? 'interrupted' : 'complete';
			const error = outcome.status === 'failed' ? String(outcome.error ?? 'generation failed') : null;

			try {
				// regenerate: ทับคำตอบเดิม ไม่ให้เหลือสองคำตอบในประวัติ
				if (body.trigger === 'regenerate-message' && body.messageId) {
					const seq = getMessageSeq(chatId, body.messageId);
					if (seq !== null) deleteMessagesFrom(chatId, seq);
				}
				saveMessages(chatId, [responseMessage], { status, modelId, error });
			} catch (cause) {
				console.error('[chat] persist failed', cause);
			}
		}
	});

	return createUIMessageStreamResponse({ stream });
};
