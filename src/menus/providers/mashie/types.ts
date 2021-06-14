import { ProviderInfo } from "../../../types";

export interface MashieMenu {
	id: string;
	title: string;
	url: string;
}

export type ListMenusResponse = MashieMenu[];

export interface MashieGeneratorOptions {
	info: ProviderInfo;
	baseUrl: string;
}

/**
 * A function that generates a provider-specific implementation of something.
 */
export type MashieGenerator<T> = (options: MashieGeneratorOptions) => T;

export type QueryMashieMenu = (id: string) => Promise<MashieMenu>;
