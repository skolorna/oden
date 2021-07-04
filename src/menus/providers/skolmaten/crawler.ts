import { DistrictsResponse, ProvincesResponse, SkolmatenStationsResponse } from "./types";
import skolmatenFetch from "./fetch";
import { ListMenus, ProviderMenu } from "../types";
import { menuTitle } from "./parser";

export function menuNameIsValid(name: string): boolean {
	return !/info/i.test(name);
}

export const listSkolmatenMenus: ListMenus = async () => {
	const { provinces } = await skolmatenFetch<ProvincesResponse>("provinces");

	const menus3d: ProviderMenu[][][] = await Promise.all(
		provinces.map(async (province) => {
			const { districts } = await skolmatenFetch<DistrictsResponse>(
				"districts",
				new URLSearchParams({
					province: province.id.toString(),
				}),
			);

			return Promise.all(
				districts.map(async (district) => {
					const { stations } = await skolmatenFetch<SkolmatenStationsResponse>(
						"stations",
						new URLSearchParams({
							district: district.id.toString(),
						}),
					);

					return stations.reduce((acc, { id, name }) => {
						if (menuNameIsValid(name)) {
							acc.push({
								id: id.toString(),
								title: menuTitle(name, district.name),
							});
						}

						return acc;
					}, [] as ProviderMenu[]);
				}),
			);
		}),
	);

	const menus = menus3d.flat(2);

	return menus;
};
