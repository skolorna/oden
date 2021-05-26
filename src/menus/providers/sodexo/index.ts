import { generateMashieProvider } from "../mashie";

const sodexo = generateMashieProvider({
	info: {
		name: "Sodexo",
		id: "sodexo",
	},
	baseUrl: "https://sodexo.mashie.com",
});

export default sodexo;
