import { redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { createProject, deleteProject, listProjects, projectChatCount } from '#lib/server/db/projects';

export const load: PageServerLoad = () => {
	return {
		projects: listProjects().map((project) => ({ ...project, chat_count: projectChatCount(project.id) }))
	};
};

export const actions: Actions = {
	create: async ({ request }) => {
		const form = await request.formData();
		const project = createProject({
			name: String(form.get('name') ?? ''),
			description: String(form.get('description') ?? ''),
			instructions: String(form.get('instructions') ?? '')
		});
		redirect(303, `/project/${project.id}`);
	},

	remove: async ({ request }) => {
		const form = await request.formData();
		const id = String(form.get('id') ?? '');
		if (id) deleteProject(id);
	}
};
