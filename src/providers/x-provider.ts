import { NotFound } from "http-errors";
import MenuID from "../menu-id";
import { Menu } from "../types";
import mpi from "./mpi";
import skolmaten from "./skolmaten";
import sodexo from "./sodexo";
import { Provider } from "./types";

export const providers: Provider[] = [skolmaten, sodexo, mpi];

/**
 * List *all* of the menus.
 */
export async function listMenus(): Promise<Menu[]> {
	const menus2d: Menu[][] = await Promise.all(
		providers.flatMap(async (provider) => {
			const providerMenus = await provider.implementation.listMenus();

			return providerMenus.map((menu) => ({
				id: new MenuID(provider.info.id, menu.id),
				title: menu.title,
			}));
		}),
	);

	return menus2d.flat();
}

export async function queryMenu(id: MenuID): Promise<Menu> {
	const provider = providers.find(({ info }) => info.id === id.provider);

	if (!provider) {
		throw new NotFound(`provider \`${id.provider}\` not found`);
	}

	const menu = await provider.implementation.queryMenu(id.providedID);

	return {
		id: new MenuID(provider.info.id, menu.id),
		title: menu.title,
	};
}
