import { redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { agentChatCount, createAgent, deleteAgent, listAgents } from '#lib/server/db/agents';
import { listModels } from '#lib/server/ai/models';

export const load: PageServerLoad = async () => {
	return {
		models: await listModels(),
		agents: listAgents().map((agent) => ({ ...agent, chat_count: agentChatCount(agent.id) }))
	};
};

export const actions: Actions = {
	create: async ({ request }) => {
		const form = await request.formData();
		const modelId = String(form.get('model_id') ?? '').trim();

		const agent = createAgent({
			name: String(form.get('name') ?? ''),
			description: String(form.get('description') ?? ''),
			systemPrompt: String(form.get('system_prompt') ?? ''),
			avatar: String(form.get('avatar') ?? ''),
			modelId: modelId === 'default' || modelId === '' ? null : modelId
		});
		redirect(303, `/agent/${agent.id}`);
	},

	remove: async ({ request }) => {
		const form = await request.formData();
		const id = String(form.get('id') ?? '');
		if (id) deleteAgent(id);
	}
};
