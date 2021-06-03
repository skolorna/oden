import fastify, { FastifyInstance, FastifyServerOptions } from "fastify";
import { IncomingMessage, Server, ServerResponse } from "http";
import Etag from "fastify-etag";
import { routes as menuRoutes } from "./menus";

const build = (opts: FastifyServerOptions = {}): FastifyInstance<Server, IncomingMessage, ServerResponse> => {
	const app: FastifyInstance = fastify({
		ignoreTrailingSlash: true,
		...opts,
	});

	app.register(Etag);

	app.get("/health", async (_, reply) => {
		return reply.header("Cache-Control", "no-cache").send("Поехали!"); // Russian for "let's go!"
	});

	app.register(menuRoutes, {
		prefix: "/menus",
	});

	return app;
};

export default build;
