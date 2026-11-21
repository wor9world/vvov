import type { PageServerLoad } from './$types';
import { listAgents } from '#lib/server/db/agents';
import { countChats, listChats } from '#lib/server/db/chats';
import { listProjects } from '#lib/server/db/projects';
import { when } from '#lib/server/format';

export const load: PageServerLoad = () => {
	const agents = listAgents();
	const projects = listProjects();

	return {
		counts: { chats: countChats(), agents: agents.length, projects: projects.length },
		recent: listChats(6).map((chat) => ({
			id: chat.id,
			title: chat.title,
			when: when(chat.updated_at),
			agent_name: chat.agent_name,
			project_name: chat.project_name
		})),
		agents: agents.slice(0, 4),
		projects: projects.slice(0, 4)
	};
};
