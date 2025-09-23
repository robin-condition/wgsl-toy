// https://stackoverflow.com/a/75717991
// https://esbuild.github.io/getting-started/#build-scripts
import * as esbuild from 'esbuild'

// esbuild
await esbuild.build({
  entryPoints: ['package.js'],
  bundle: true,
  outfile: 'src/package.js',
  format: 'esm',
  minify: true,
}).catch(() => process.exit(1));