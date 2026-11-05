// #420: responsive entries cannot silently omit either required reference width.
const assert=require('node:assert/strict');
const specs=require('./design_parity/spec.cjs');
assert(specs.length>100,'the real approved-design specification is loaded');
for(const entry of specs)for(const width of [840,1280])assert(!entry.widths||entry.widths.includes(width),`${entry.name} excludes ${width}`);
console.log('Every responsive specification entry permits both 840 and 1280');
