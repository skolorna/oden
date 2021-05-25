import { UnprocessableEntity } from "http-errors";

export function toSkolmatenID(input: string): number {
	const parsed = parseInt(input, 10);

	if (!Number.isInteger(parsed) || parsed.toString() !== input) {
		throw new UnprocessableEntity(`menu id must be an integer (got \`${input}\`)`);
	}

	return parsed;
}
