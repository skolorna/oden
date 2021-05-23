import { generateMashieImplementation } from "../mashie";
import { Provider } from "../types";

const mpi: Provider = {
	info: {
		name: "MPI",
		id: "mpi",
	},
	implementation: generateMashieImplementation("https://mpi.mashie.com"),
};

export default mpi;
