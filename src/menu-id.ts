import { UnprocessableEntity } from "http-errors";

const SEGMENT_SEPARATOR = ".";

export default class MenuID {
	constructor(public provider: string, public providedID: string) {}

	public toString(): string {
		return `${this.provider}${SEGMENT_SEPARATOR}${this.providedID}`;
	}

	public toJSON(): string {
		return this.toString();
	}

	public static parse(input: string): MenuID {
		const segments = input.split(SEGMENT_SEPARATOR);

		if (segments.length !== 2) {
			throw new UnprocessableEntity("invalid menu id");
		}

		const [provider, providedID] = segments;

		return new MenuID(provider, providedID);
	}
}
