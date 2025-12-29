import { defineConfig } from "vite";

// https://vite.dev/config/
export default defineConfig({
	base: "app://app/",
	build: {
		outDir: "../../examples/files-cors",
	},
});
