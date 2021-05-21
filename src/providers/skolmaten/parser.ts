import { UnprocessableEntity } from "http-errors";
import { SchoolID } from "../../types";

export function toSkolmatenID(input: SchoolID): number {
	const parsed = parseInt(input, 10);

	if (!Number.isInteger(parsed) || parsed.toString() !== input) {
		throw new UnprocessableEntity(`school id must be an integer (got \`${input}\`)`);
	}

	return parsed;
}
