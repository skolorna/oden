import { FastifyPluginCallback } from "fastify";
import { Provider } from "./types";

export function generateProviderRoutes({ info, implementation }: Provider): FastifyPluginCallback {
	return async (fastify) => {
		fastify.get("/", async () => {
			return info;
		});

		fastify.get("/schools", async () => {
			const schools = await implementation.listSchools();

			return schools;
		});
	};
}
