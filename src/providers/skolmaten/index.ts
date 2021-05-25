import { Provider } from "../types";
import { getSkolmatenSchools } from "./crawler";
import { listSkolmatenDays } from "./days";
import { querySkolmatenMenu } from "./menu";

const skolmaten: Provider = {
	info: {
		name: "Skolmaten",
		id: "skolmaten",
	},
	implementation: {
		listMenus: getSkolmatenSchools,
		queryMenu: querySkolmatenMenu,
		listDays: listSkolmatenDays,
	},
};

export default skolmaten;
