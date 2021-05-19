import fastify, { FastifyInstance, FastifyServerOptions } from "fastify";
import { IncomingMessage, Server, ServerResponse } from "http";

const build = (opts: FastifyServerOptions = {}): FastifyInstance<Server, IncomingMessage, ServerResponse> => {
	const app: FastifyInstance = fastify(opts);

	app.get("/health", async () => {
		return "Поехали!"; // Russian for "let's go!"
	});

	return app;
};

export default build;
