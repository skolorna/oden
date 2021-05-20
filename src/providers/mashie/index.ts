import { Provider } from "../types";
import { getMashieMenu } from "./menu";
import { listMashieSchools } from "./schools";

const mashie: Provider = {
	info: {
		name: "Sodexo (Mashie)",
		id: "mashie",
	},
	implementation: {
		getSchools: listMashieSchools,
		getMenu: getMashieMenu,
	},
};

export default mashie;
