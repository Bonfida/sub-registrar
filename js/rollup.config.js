import typescript from "@rollup/plugin-typescript";
import commonjs from "@rollup/plugin-commonjs";
import terser from "@rollup/plugin-terser";
import { nodeResolve } from "@rollup/plugin-node-resolve";
import replace from "@rollup/plugin-replace";
import babel from "@rollup/plugin-babel";

export default {
  input: "src/index.ts",
  output: [
    {
      file: "dist/index.mjs",
      format: "esm",
    },
    { file: "dist/index.cjs", format: "cjs" },
  ],
  external: ["@solana/web3.js"],
  plugins: [
    nodeResolve({ browser: true, preferBuiltins: false }),
    typescript(),
    commonjs(),
    babel({ babelHelpers: "bundled" }),
    replace({
      "process.env.NODE_ENV": JSON.stringify("production"),
      preventAssignment: false,
    }),
    terser(),
  ],
  onwarn: function (warning, handler) {
    if (warning.code === "THIS_IS_UNDEFINED") return;
    handler(warning);
  },
};
