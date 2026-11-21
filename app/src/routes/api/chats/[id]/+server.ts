import { error } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { deleteChat, getChat, setChatModel } from '#lib/server/db/chats';

type CleanModel = { ok: true; value: string | null } | { ok: false };

/** กันค่าแปลกปลอมจาก client — รับเฉพาะรูปแบบ model id ที่ปลอดภัย */
function cleanModelId(value: unknown): CleanModel {
	if (value === null || value === undefined) return { ok: true, value: null };
	if (typeof value !== 'string') return { ok: false };

	const id = value.trim();
	if (!id) return { ok: true, value: null };
	if (id.length > 120 || /\s/.test(id)) return { ok: false };
	if (!/^[A-Za-z0-9._:\/-]+$/.test(id)) return { ok: false };

	return { ok: true, value: id };
}

export const PATCH: RequestHandler = async ({ params, request }) => {
	if (!/^[a-z0-9-]{1,64}$/i.test(params.id)) error(404, 'No such chat');
	if (!getChat(params.id)) error(404, 'No such chat');

	let body: unknown;
	try {
		body = await request.json();
	} catch {
		return Response.json({ message: 'Expected a JSON body.' }, { status: 400 });
	}

	const input = body && typeof body === 'object' ? (body as Record<string, unknown>) : {};
	const model = cleanModelId(input.modelId);
	if (!model.ok) {
		return Response.json({ message: 'modelId must be a string or null.' }, { status: 400 });
	}

	setChatModel(params.id, model.value);
	return Response.json({ id: params.id, modelId: model.value });
};

export const DELETE: RequestHandler = ({ params }) => {
	if (!/^[a-z0-9-]{1,64}$/i.test(params.id)) error(404, 'No such chat');
	if (!getChat(params.id)) error(404, 'No such chat');

	deleteChat(params.id);
	return new Response(null, { status: 204 });
};
