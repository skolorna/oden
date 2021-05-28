import { Provider } from "../types";
import { listSkolmatenMenus } from "./crawler";
import { listSkolmatenDays } from "./days";
import { querySkolmatenMenu } from "./menu";

const skolmaten: Provider = {
	info: {
		name: "Skolmaten",
		id: "skolmaten",
	},
	implementation: {
		listMenus: listSkolmatenMenus,
		queryMenu: querySkolmatenMenu,
		listDays: listSkolmatenDays,
	},
};

export default skolmaten;
