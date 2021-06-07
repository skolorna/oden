import { NotFound } from "http-errors";
import { URL } from "url";
import { fetchRetry } from "../../../utils/fetch-retry";
import { ListMenus, QueryMenu } from "../types";
import { ListMenusResponse, MashieGenerator, QueryMashieMenu } from "./types";

const getMenuLister: MashieGenerator<() => Promise<ListMenusResponse>> = ({ baseUrl }) => {
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

export const getMashieSchoolLister: MashieGenerator<ListMenus> = (options) => {
	const listSchools = getMenuLister(options);

	return async () => {
		const schools = await listSchools();

		return schools.map(({ id, title }) => ({
			id,
			title,
		}));
	};
};

export const getRawMashieMenuQuerier: MashieGenerator<QueryMashieMenu> = (baseUrl) => {
	const listSchools = getMenuLister(baseUrl);

	return async (id) => {
		const menus = await listSchools();

		const menu = menus.find(({ id: menuID }) => menuID === id);

		if (!menu) {
			throw new NotFound(`menu with id \`${id}\` not found`);
		}

		return menu;
	};
};

export const getMashieMenuQuerier: MashieGenerator<QueryMenu> = (options) => {
	const queryMenu = getRawMashieMenuQuerier(options);

	return async (queryId) => {
		const { title, id } = await queryMenu(queryId);

		return {
			title,
			id,
		};
	};
};
