import { NotFound } from "http-errors";
import MenuID from "../menu-id";
import { Menu } from "../types";
import mpi from "./providers/mpi";
import skolmaten from "./providers/skolmaten";
import sodexo from "./providers/sodexo";
import { Provider } from "./providers/types";

export const providers: Provider[] = [skolmaten, sodexo, mpi];

/**
 * Safely get a provider by its id (the function throws an error if no provider is found).
 *
 * @param {string} id ID of the provider.
 *
 * @returns {Provider} The provider.
 */
export function getProviderByID(id: string): Provider {
	const provider = providers.find(({ info }) => info.id === id);

	if (!provider) {
		throw new NotFound(`provider with id \`${id}\` not found`);
	}

	return provider;
}

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
				provider: provider.info,
			}));
		}),
	);

	return menus2d.flat();
}

export async function queryMenu(id: MenuID): Promise<Menu> {
	const provider = getProviderByID(id.provider);

	const menu = await provider.implementation.queryMenu(id.providedID);

	return {
		id: new MenuID(provider.info.id, menu.id),
		title: menu.title,
		provider: provider.info,
	};
}
