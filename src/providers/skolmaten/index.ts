import { Provider } from "../types";
import { getSkolmatenSchools } from "./crawler";
import { getSkolmatenMenu } from "./menu";

const skolmaten: Provider = {
	info: {
		name: "Skolmaten",
		id: "skolmaten",
	},
	implementation: {
		getSchools: getSkolmatenSchools,
		getMenu: getSkolmatenMenu,
	},
};

export default skolmaten;
