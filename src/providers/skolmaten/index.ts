import { Provider } from "../types";
import { getSkolmatenSchools } from "./crawler";
import { getSkolmatenMenu } from "./menu";

const skolmaten: Provider = {
	info: {
		name: "Skolmaten",
		id: "skolmaten",
	},
	implementation: {
		listSchools: getSkolmatenSchools,
		getMenu: getSkolmatenMenu,
	},
};

export default skolmaten;
