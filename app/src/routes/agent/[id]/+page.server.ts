import { error, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { deleteAgent, getAgent, updateAgent } from '#lib/server/db/agents';
import { createChat, listChatsForAgent } from '#lib/server/db/chats';
import { listModels } from '#lib/server/ai/models';
import { when } from '#lib/server/format';

export const load: PageServerLoad = async ({ params }) => {
	const agent = getAgent(params.id);
	if (!agent) error(404, 'Agent not found');

	return {
		agent,
		models: await listModels(),
		chats: listChatsForAgent(agent.id).map((chat) => ({ ...chat, when: when(chat.updated_at) }))
	};
};

export const actions: Actions = {
	newChat: ({ params }) => {
		const chat = createChat({ agentId: params.id });
		redirect(303, `/chat/${chat.id}`);
	},

	update: async ({ params, request }) => {
		const form = await request.formData();
		const modelId = String(form.get('model_id') ?? '').trim();

		updateAgent(params.id, {
			name: String(form.get('name') ?? ''),
			description: String(form.get('description') ?? ''),
			systemPrompt: String(form.get('system_prompt') ?? ''),
			avatar: String(form.get('avatar') ?? ''),
			modelId: modelId === 'default' || modelId === '' ? null : modelId
		});
	},

	remove: ({ params }) => {
		deleteAgent(params.id);
		redirect(303, '/agent');
	}
};
