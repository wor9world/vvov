import { error } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { getAgent } from '#lib/server/db/agents';
import { getChat } from '#lib/server/db/chats';
import { getProject } from '#lib/server/db/projects';
import { listMessages } from '#lib/server/db/messages';
import { listModels } from '#lib/server/ai/models';

export const load: PageServerLoad = async ({ params }) => {
	const chat = getChat(params.id);
	if (!chat) error(404, 'Chat not found');

	const agent = chat.agent_id ? getAgent(chat.agent_id) : null;
	const project = chat.project_id ? getProject(chat.project_id) : null;

	return {
		chat,
		messages: listMessages(chat.id),
		agentName: agent?.name ?? null,
		projectName: project?.name ?? null,
		models: await listModels()
	};
};
