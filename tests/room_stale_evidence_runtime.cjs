// GitHub #794: a newly opened Room clears evidence surfaces before requesting its snapshot.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const source = fs.readFileSync('src/ui/room.js', 'utf8');
assert.match(source, /function resetReviewEvidence\(\)[\s\S]*?\$\('review-summary'\)\.replaceChildren\(\)[\s\S]*?\$\('test-results'\)\.replaceChildren\(\)[\s\S]*?\$\('approve'\)\.disabled=true;\$\('merge'\)\.disabled=true;/, 'prior review and approval surfaces are cleared');
assert.match(source, /resetReviewEvidence\(\);\s*let snapshot/, 'clear evidence synchronously before the new Room snapshot is loaded');
assert.match(source, /async function loadReview\(\)\s*\{\s*try\{const next=await request\(\{kind:'evidence'\}\);if\(roomStopped\)return;/, 'a stopped Room ignores late evidence responses');
console.log('room evidence reset: stale evidence cleared before loading another run');
