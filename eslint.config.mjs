import typescriptEslint from "@typescript-eslint/eslint-plugin";
import prettier from "eslint-plugin-prettier";
import globals from "globals";
import tsParser from "@typescript-eslint/parser";
import path from "node:path";
import { fileURLToPath } from "node:url";
import js from "@eslint/js";
import { FlatCompat } from "@eslint/eslintrc";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const compat = new FlatCompat({
	baseDirectory: __dirname,
	recommendedConfig: js.configs.recommended,
	allConfig: js.configs.all
});

const eslintPluginPrettierRecommended = require("eslint-plugin-prettier/recommended");

module.exports = {
	eslintPluginPrettierRecommended
}

export default [...compat.extends(
	"eslint:recommended",
	"plugin:@typescript-eslint/recommended",
), {
	plugins: {
		"@typescript-eslint": typescriptEslint,
		prettier,
	},

	languageOptions: {

		globals: {
			...globals.browser,
		},

		parser: tsParser,
		ecmaVersion: "latest",
		sourceType: "module",
	},

	rules: {
		indent: ["error", "tab"],
		"linebreak-style": ["error", "unix"],
		quotes: ["error", "double"],
		semi: ["error", "always"],
	},
}];