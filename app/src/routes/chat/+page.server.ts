import type { Actions, PageServerLoad } from './$types';
import { deleteChat, listChats, renameChat } from '#lib/server/db/chats';
import { when } from '#lib/server/format';

export const load: PageServerLoad = () => {
	return {
		chats: listChats(100).map((chat) => ({ ...chat, when: when(chat.updated_at) }))
	};
};

export const actions: Actions = {
	rename: async ({ request }) => {
		const form = await request.formData();
		const id = String(form.get('id') ?? '');
		const title = String(form.get('title') ?? '');
		if (id) renameChat(id, title);
	},

	remove: async ({ request }) => {
		const form = await request.formData();
		const id = String(form.get('id') ?? '');
		if (id) deleteChat(id);
	}
};
