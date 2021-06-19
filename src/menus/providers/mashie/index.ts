import { Provider } from "../types";
import { getMashieDayLister } from "./days";
import { getMashieMenuLister, getMashieMenuQuerier } from "./menus";
import { MashieFactory } from "./types";

export const generateMashieProvider: MashieFactory<Provider> = (options) => {
	return {
		info: options.info,
		implementation: {
			listDays: getMashieDayLister(options),
			listMenus: getMashieMenuLister(options),
			queryMenu: getMashieMenuQuerier(options),
		},
	};
};
