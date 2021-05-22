import { UnprocessableEntity } from "http-errors";
import { DateTime } from "luxon";

export function parseISO8601(input: string): DateTime {
	const timestamp = DateTime.fromISO(input);

	if (!timestamp.isValid) {
		throw new UnprocessableEntity(timestamp.invalidReason ?? `\`${input}\` is not a valid ISO8601 timestamp`);
	}

	return timestamp;
}
