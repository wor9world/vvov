import type { ContextSummary, WorkspaceCounts } from '#lib/context';
import { countChats, getChat } from './db/chats';
import { getAgent, agentChatCount, listAgents } from './db/agents';
import { getProject, projectChatCount, listProjects } from './db/projects';
import { countMessages } from './db/messages';

export function readCounts(): WorkspaceCounts {
	return {
		chats: countChats(),
		agents: listAgents().length,
		projects: listProjects().length
	};
}

/** สร้างบริบทจาก pathname — ใช้ใน layout load เท่านั้น (ฝั่ง server) */
export function buildContext(pathname: string, counts: WorkspaceCounts): ContextSummary {
	const chat = pathname.match(/^\/chat\/([a-z0-9-]{1,64})$/i);
	if (chat) {
		const row = getChat(chat[1]);
		if (row) {
			return {
				kind: 'chat',
				id: row.id,
				title: row.title,
				modelId: row.model_id,
				agentName: row.agent_id ? (getAgent(row.agent_id)?.name ?? null) : null,
				projectName: row.project_id ? (getProject(row.project_id)?.name ?? null) : null,
				messageCount: countMessages(row.id)
			};
		}
	}

	const agent = pathname.match(/^\/agent\/([a-z0-9-]{1,64})$/i);
	if (agent) {
		const row = getAgent(agent[1]);
		if (row) {
			return {
				kind: 'agent',
				id: row.id,
				name: row.name,
				description: row.description,
				modelId: row.model_id,
				chatCount: agentChatCount(row.id)
			};
		}
	}

	const project = pathname.match(/^\/project\/([a-z0-9-]{1,64})$/i);
	if (project) {
		const row = getProject(project[1]);
		if (row) {
			return {
				kind: 'project',
				id: row.id,
				name: row.name,
				description: row.description,
				chatCount: projectChatCount(row.id)
			};
		}
	}

	return { kind: 'workspace', counts };
}
