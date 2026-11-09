// Resolve existing dependencies from this checkout, including a shared parent node_modules.
// Resolution only: never installs or invokes a package manager.
const path = require('node:path');
const { root } = require('./catalog.cjs');
const entries = { typescript: 'bin/tsc', vite: 'bin/vite.js' };
const name = process.argv[2];
if (!Object.hasOwn(entries, name)) throw Error('Expected typescript or vite');
const packageFile = require.resolve(`${name}/package.json`, { paths: [root] });
console.log(require.resolve(path.join(path.dirname(packageFile), entries[name])));
