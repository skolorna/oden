import { Provider } from "../types";
import { getMashieDayLister } from "./days";
import { getMashieSchoolLister, getMashieMenuQuerier } from "./menus";
import { MashieGenerator } from "./types";

export const generateMashieProvider: MashieGenerator<Provider> = (options) => {
	return {
		info: options.info,
		implementation: {
			listDays: getMashieDayLister(options),
			listMenus: getMashieSchoolLister(options),
			queryMenu: getMashieMenuQuerier(options),
		},
	};
};
