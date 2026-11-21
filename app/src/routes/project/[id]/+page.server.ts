import { error, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { deleteProject, getProject, updateProject } from '#lib/server/db/projects';
import { createChat, listChatsForProject } from '#lib/server/db/chats';
import { when } from '#lib/server/format';

export const load: PageServerLoad = ({ params }) => {
	const project = getProject(params.id);
	if (!project) error(404, 'Project not found');

	return {
		project,
		chats: listChatsForProject(project.id).map((chat) => ({ ...chat, when: when(chat.updated_at) }))
	};
};

export const actions: Actions = {
	newChat: ({ params }) => {
		const chat = createChat({ projectId: params.id });
		redirect(303, `/chat/${chat.id}`);
	},

	update: async ({ params, request }) => {
		const form = await request.formData();
		updateProject(params.id, {
			name: String(form.get('name') ?? ''),
			description: String(form.get('description') ?? ''),
			instructions: String(form.get('instructions') ?? '')
		});
	},

	remove: ({ params }) => {
		deleteProject(params.id);
		redirect(303, '/project');
	}
};
