import { generateMashieProvider } from "../mashie";

const mpi = generateMashieProvider({
	info: {
		name: "MPI",
		id: "mpi",
	},
	baseUrl: "https://mpi.mashie.com",
});

export default mpi;
