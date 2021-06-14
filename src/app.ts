import fastify, { FastifyInstance, FastifyServerOptions } from "fastify";
import { IncomingMessage, Server, ServerResponse } from "http";
import Etag from "fastify-etag";
import fastifyCors from "fastify-cors";
import fastifyCompress from "fastify-compress";
import { routes as menuRoutes } from "./menus";

const build = (opts: FastifyServerOptions = {}): FastifyInstance<Server, IncomingMessage, ServerResponse> => {
	const app: FastifyInstance = fastify({
		ignoreTrailingSlash: true,
		...opts,
	});

	app.register(fastifyCors, {
		origin: "*",
	});

	app.register(Etag);

	app.register(fastifyCompress);

	app.get("/health", async (_, reply) => {
		return reply.header("Cache-Control", "no-cache").send("Поехали!"); // Russian for "let's go!"
	});

	app.register(menuRoutes, {
		prefix: "/menus",
	});

	return app;
};

export default build;
