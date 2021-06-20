import { NotFound } from "http-errors";
import { URL } from "url";
import { fetchRetry } from "../../../utils/fetch-retry";
import { ListMenus, QueryMenu } from "../types";
import { ListMenusResponse, MashieFactory, QueryMashieMenu } from "./types";

const getMenuLister: MashieFactory<() => Promise<ListMenusResponse>> = ({ baseUrl }) => {
	return async () => {
		const data: ListMenusResponse = await fetchRetry(
			new URL("/public/app/internal/execute-query?country=se", baseUrl),
			{
				method: "POST",
			},
		).then((res) => res.json());

		return data;
	};
};

export const getMashieMenuLister: MashieFactory<ListMenus> = (options) => {
	const listMenus = getMenuLister(options);

	return async () => {
		const menus = await listMenus();

		return menus.map(({ id, title }) => ({
			id,
			title,
		}));
	};
};

export const getRawMashieMenuQuerier: MashieFactory<QueryMashieMenu> = (baseUrl) => {
	const listMenus = getMenuLister(baseUrl);

	return async (id) => {
		const menus = await listMenus();

		const menu = menus.find(({ id: menuID }) => menuID === id);

		if (!menu) {
			throw new NotFound(`menu with id \`${id}\` not found`);
		}

		return menu;
	};
};

export const getMashieMenuQuerier: MashieFactory<QueryMenu> = (options) => {
	const queryMenu = getRawMashieMenuQuerier(options);

	return async (queryId) => {
		const { title, id } = await queryMenu(queryId);

		return {
			title,
			id,
		};
	};
};
