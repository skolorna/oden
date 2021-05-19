module.exports = {
	preset: "ts-jest",
	testEnvironment: "node",
	collectCoverageFrom: ["src/**/*.ts"],
	globals: {
		"ts-jest": {
			tsconfig: "tests/tsconfig.json",
		},
	},
	testPathIgnorePatterns: ["/node_modules/", "/dist/"],
};
