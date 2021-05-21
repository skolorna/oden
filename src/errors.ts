/* eslint-disable max-classes-per-file */

export class ParseError extends Error {
	constructor(message?: string) {
		super(message);
		this.name = "ParseError";
	}
}
