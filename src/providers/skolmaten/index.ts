import { Provider } from "../types";
import { getSkolmatenSchools } from "./crawler";
import { getSkolmatenMenu } from "./menu";
import { querySkolmatenSchool } from "./school";

const skolmaten: Provider = {
	info: {
		name: "Skolmaten",
		id: "skolmaten",
	},
	implementation: {
		listSchools: getSkolmatenSchools,
		querySchool: querySkolmatenSchool,
		getMenu: getSkolmatenMenu,
	},
};

export default skolmaten;
