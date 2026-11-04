// Uso: node tools/license-issuer/cli.mjs <private.key> <e-mail> <monthly|yearly>
import { readFileSync } from 'node:fs';

import { issueLicense } from './issuer.mjs';

const [file, email, plan] = process.argv.slice(2);
if (!file || !email || !plan) {
  console.error('uso: node cli.mjs <private.key> <e-mail> <monthly|yearly>');
  process.exit(1);
}
console.log(await issueLicense({ privateKey: readFileSync(file, 'utf8'), email, plan }));
