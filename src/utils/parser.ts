import { UnprocessableEntity } from "http-errors";
import { LocalDate } from "js-joda";

export function parseISODate(input: string): LocalDate {
	try {
		return LocalDate.parse(input);
	} catch (error) {
		throw new UnprocessableEntity(error.message);
	}
}
