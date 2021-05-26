import { Static, Type } from "@sinclair/typebox";

export const QueryMenuParams = Type.Object({
	menuId: Type.String(),
});

export type QueryMenuParamsType = Static<typeof QueryMenuParams>;

export const ListDaysQuery = Type.Object({
	first: Type.Optional(Type.String()),
	last: Type.Optional(Type.String()),
});

export type ListDaysQueryType = Static<typeof ListDaysQuery>;
