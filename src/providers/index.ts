import { FastifyPluginCallback } from "fastify";
import { generateProviderRoutes } from "./provider-routes";
import skolmaten from "./skolmaten";
import sodexo from "./sodexo";
import mpi from "./mpi";
import { Provider } from "./types";

export const providers: Provider[] = [skolmaten, sodexo, mpi];

export const providerInfo = providers.map((provider) => provider.info);

export const routes: FastifyPluginCallback = async (fastify) => {
	providers.forEach((provider) => {
		fastify.register(generateProviderRoutes(provider), {
			prefix: `/${provider.info.id}`,
		});
	});

	fastify.get("/", async () => {
		return providerInfo;
	});
};
