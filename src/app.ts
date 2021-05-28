import fastify, { FastifyInstance, FastifyServerOptions } from "fastify";
import { IncomingMessage, Server, ServerResponse } from "http";
import { routes as menuRoutes } from "./menus";

const build = (opts: FastifyServerOptions = {}): FastifyInstance<Server, IncomingMessage, ServerResponse> => {
	const app: FastifyInstance = fastify({
		ignoreTrailingSlash: true,
		...opts,
	});

	app.get("/health", async () => {
		return "Поехали!"; // Russian for "let's go!"
	});

	app.register(menuRoutes, {
		prefix: "/menus",
	});

	return app;
};

export default build;
