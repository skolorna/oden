import { generateMashieImplementation } from "../mashie";
import { Provider } from "../types";

const sodexo: Provider = {
	info: {
		name: "Sodexo",
		id: "sodexo",
	},
	implementation: generateMashieImplementation("https://sodexo.mashie.com"),
};

export default sodexo;
