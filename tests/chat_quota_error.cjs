// GitHub #664: a coordinator chat that fails on a usage limit says so with the reset time and points at another coordinator,
// instead of printing the raw CLI event lines.
const fs = require('node:fs'), vm = require('node:vm'), assert = require('node:assert/strict');
const source = fs.readFileSync('src/ui/chat.js', 'utf8');
const start = source.indexOf('function chatFailure'), end = source.indexOf('function settle');
assert.ok(start > 0 && end > start, 'chatFailure exists');
const context = vm.createContext({ acMessage: (id, v = {}) => id + (v.value1 ? ':' + v.value1 : '') });
vm.runInContext(source.slice(start, end) + ';this.chatFailure=chatFailure;', context);
const raw = 'agentconductor: coordinator failed (exit code 1): {"type":"thread.started","thread_id":"x"}\n{"type":"error","message":"You’ve hit your usage limit. Upgrade to Pro (https://chatgpt.com/explore/pro), visit https://chatgpt.com/settings/usage to purchase more credits or try again at Nov 11th, 2026 2:31 AM."}\n{"type":"turn.failed","error":{"message":"You’ve hit your usage limit. try again at Nov 11th, 2026 2:31 AM."}}\nlogs: X';
assert.equal(context.chatFailure(raw), 'startup.chat_quota_error:startup.chat_quota_resets:Nov 11th, 2026 2:31 AM');
assert.equal(context.chatFailure('Claude usage limit reached'), 'startup.chat_quota_error', 'no reset time: the message stays without one');
assert.equal(context.chatFailure('no coordinator'), 'startup.chat_error:no coordinator', 'other failures keep the plain message');
console.log('chat quota error: usage limit named with its reset time; other errors unchanged');
