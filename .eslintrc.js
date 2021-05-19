module.exports = {
	parser: "@typescript-eslint/parser",
	parserOptions: {
		ecmaVersion: 2020,
		sourceType: "module",
	},
	extends: [
		"plugin:@typescript-eslint/recommended",
		"airbnb-typescript/base",
		"plugin:prettier/recommended",
		"plugin:jest/recommended",
	],
	parserOptions: {
		project: ["./src/tsconfig.json", "./tests/tsconfig.json"],
	},
	rules: {
		"import/prefer-default-export": "off",
	},
	ignorePatterns: ["*.js", "dist"],
};
