import type { RequestHandler } from './$types';
import { draftAgent, draftProject } from '#lib/server/ai/draft';

interface DraftBody {
	kind?: 'agent' | 'project';
	prompt?: string;
}

export const POST: RequestHandler = async ({ request }) => {
	let body: DraftBody;
	try {
		body = await request.json();
	} catch {
		return Response.json({ error: 'invalid JSON body' }, { status: 400 });
	}

	const prompt = String(body.prompt ?? '').trim();
	if (!prompt) return Response.json({ error: 'prompt is required' }, { status: 400 });
	if (prompt.length > 2000) return Response.json({ error: 'prompt too long (max 2000)' }, { status: 400 });

	try {
		const draft = body.kind === 'project' ? await draftProject(prompt) : await draftAgent(prompt);
		return Response.json(draft);
	} catch (cause) {
		console.error('[draft] failed', cause);
		return Response.json({ error: String(cause) }, { status: 502 });
	}
};
