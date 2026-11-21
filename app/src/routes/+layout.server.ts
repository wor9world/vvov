import type { LayoutServerLoad } from './$types';
import { listChats } from '#lib/server/db/chats.ts';
import { buildContext, readCounts } from '#lib/server/context.ts';
import { chatGroup } from '#lib/context.ts';

export const load: LayoutServerLoad = ({ url }) => {
	const counts = readCounts();

	return {
		chats: listChats(80).map((chat) => ({
			id: chat.id,
			title: chat.title,
			updated_at: chat.updated_at,
			group: chatGroup(chat.updated_at)
		})),
		counts,
		context: buildContext(url.pathname, counts)
	};
};
