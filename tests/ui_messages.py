"""Helpers for testing ID-based UI messages with the production runtime."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

def catalog(language="en"):
    return json.loads((ROOT / f"src/ui/messages/{language}.json").read_text(encoding="utf-8"))

def translated_message(english):
    en, ko = catalog(), catalog("ko")
    return any((text == english or (english.endswith(" ") and text.startswith(english)))
               and ko.get(key) != text for key, text in en.items())

def js_prelude():
    # Small fixture DOM; formatting/bindings come from the actual shipped runtime.
    return r"""
const __i18nVm=require('node:vm'),__i18nFs=require('node:fs');
const __i18nDocument={body:{childNodes:[]},documentElement:{lang:'en'},dispatchEvent(){},createTextNode(text){return {nodeType:3,childNodes:[],textContent:text};}};
const AC_MESSAGES={en:require('./src/ui/messages/en.json'),ko:require('./src/ui/messages/ko.json')};
const __i18nRuntime=__i18nVm.runInNewContext(__i18nFs.readFileSync('src/ui/i18n.js','utf8').split('// DD-111:')[0]+';({AC_I18N,acMessage})',{document:__i18nDocument,AC_MESSAGES,Event:class{}});
const acMessage=__i18nRuntime.acMessage;
const AC_I18N={...__i18nRuntime.AC_I18N,bind(node,value,attr){if(attr&&!node.getAttribute){node.getAttribute=k=>node[k];node.setAttribute=(k,v)=>node[k]=v;}return __i18nRuntime.AC_I18N.bind(node,value,attr);}};
"""
