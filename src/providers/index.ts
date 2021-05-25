import { FastifyPluginCallback } from "fastify";
import { generateProviderRoutes } from "./provider-routes";
import { providers } from "./x-provider";

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
