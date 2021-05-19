import app from "./app";

const { PORT = 8000 } = process.env;

const server = app({
	logger: {
		level: "info",
		prettyPrint: true,
	},
});

server.listen(PORT, (error) => {
	if (error) {
		// eslint-disable-next-line no-console
		console.error(error);
		process.exit(1);
	}
});
