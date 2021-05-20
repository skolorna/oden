import fastify, { FastifyInstance, FastifyServerOptions } from "fastify";
import { IncomingMessage, Server, ServerResponse } from "http";
import { routes as providerRoutes } from "./providers";

const build = (opts: FastifyServerOptions = {}): FastifyInstance<Server, IncomingMessage, ServerResponse> => {
	const app: FastifyInstance = fastify(opts);

	app.get("/health", async () => {
		return "Поехали!"; // Russian for "let's go!"
	});

	app.register(providerRoutes, {
		prefix: "/providers",
	});

	return app;
};

export default build;
