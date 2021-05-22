import { Static, Type } from "@sinclair/typebox";

export const QuerySchoolParams = Type.Object({
	schoolId: Type.String(),
});

export type QuerySchoolParamsType = Static<typeof QuerySchoolParams>;

export const GetMenuQuery = Type.Object({
	first: Type.Optional(Type.String()),
	last: Type.Optional(Type.String()),
});

export type GetMenuQueryType = Static<typeof GetMenuQuery>;
