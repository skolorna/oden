import app from "./app";

const { PORT = 8000, ADDRESS = "0.0.0.0" } = process.env;

const server = app({
	logger: {
		level: "info",
		prettyPrint: true,
	},
});

server.listen(PORT, ADDRESS, (error) => {
	if (error) {
		// eslint-disable-next-line no-console
		console.error(error);
		process.exit(1);
	}
});
