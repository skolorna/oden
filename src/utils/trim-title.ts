/**
 * Remove excess whitespace and punctuation.
 *
 * @param {string} input Ugly title.
 *
 * @returns {string} Polished title.
 */
export function trimTitle(value: string): string {
	// Trim same characters as String.trim(), as well as punctuation.
	// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/Trim#polyfill
	return value.replace(/^[\s\uFEFF\xA0.]+|[\s\uFEFF\xA0.]+$/g, "").replace(/[\s\uFEFF\xA0]+/g, " ");
}
