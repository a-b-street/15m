var Me=Object.defineProperty;var Te=(R,k,A)=>k in R?Me(R,k,{enumerable:!0,configurable:!0,writable:!0,value:A}):R[k]=A;var _e=(R,k,A)=>(Te(R,typeof k!="symbol"?k+"":k,A),A);(function(){"use strict";/**
 * @license
 * Copyright 2019 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */const R=Symbol("Comlink.proxy"),k=Symbol("Comlink.endpoint"),A=Symbol("Comlink.releaseProxy"),H=Symbol("Comlink.finalizer"),z=Symbol("Comlink.thrown"),Z=n=>typeof n=="object"&&n!==null||typeof n=="function",ie={canHandle:n=>Z(n)&&n[R],serialize(n){const{port1:e,port2:t}=new MessageChannel;return J(n,e),[t,[t]]},deserialize(n){return n.start(),ue(n)}},ae={canHandle:n=>Z(n)&&z in n,serialize({value:n}){let e;return n instanceof Error?e={isError:!0,value:{message:n.message,name:n.name,stack:n.stack}}:e={isError:!1,value:n},[e,[]]},deserialize(n){throw n.isError?Object.assign(new Error(n.value.message),n.value):n.value}},Y=new Map([["proxy",ie],["throw",ae]]);function se(n,e){for(const t of n)if(e===t||t==="*"||t instanceof RegExp&&t.test(e))return!0;return!1}function J(n,e=globalThis,t=["*"]){e.addEventListener("message",function r(o){if(!o||!o.data)return;if(!se(t,o.origin)){console.warn(`Invalid origin '${o.origin}' for comlink proxy`);return}const{id:b,type:c,path:u}=Object.assign({path:[]},o.data),f=(o.data.argumentList||[]).map(S);let s;try{const g=u.slice(0,-1).reduce((m,I)=>m[I],n),y=u.reduce((m,I)=>m[I],n);switch(c){case"GET":s=y;break;case"SET":g[u.slice(-1)[0]]=S(o.data.value),s=!0;break;case"APPLY":s=y.apply(g,f);break;case"CONSTRUCT":{const m=new y(...f);s=we(m)}break;case"ENDPOINT":{const{port1:m,port2:I}=new MessageChannel;J(n,I),s=ge(m,[m])}break;case"RELEASE":s=void 0;break;default:return}}catch(g){s={value:g,[z]:0}}Promise.resolve(s).catch(g=>({value:g,[z]:0})).then(g=>{const[y,m]=U(g);e.postMessage(Object.assign(Object.assign({},y),{id:b}),m),c==="RELEASE"&&(e.removeEventListener("message",r),K(e),H in n&&typeof n[H]=="function"&&n[H]())}).catch(g=>{const[y,m]=U({value:new TypeError("Unserializable return value"),[z]:0});e.postMessage(Object.assign(Object.assign({},y),{id:b}),m)})}),e.start&&e.start()}function ce(n){return n.constructor.name==="MessagePort"}function K(n){ce(n)&&n.close()}function ue(n,e){return V(n,[],e)}function C(n){if(n)throw new Error("Proxy has been released and is not useable")}function X(n){return x(n,{type:"RELEASE"}).then(()=>{K(n)})}const P=new WeakMap,W="FinalizationRegistry"in globalThis&&new FinalizationRegistry(n=>{const e=(P.get(n)||0)-1;P.set(n,e),e===0&&X(n)});function fe(n,e){const t=(P.get(e)||0)+1;P.set(e,t),W&&W.register(n,e,n)}function be(n){W&&W.unregister(n)}function V(n,e=[],t=function(){}){let r=!1;const o=new Proxy(t,{get(b,c){if(C(r),c===A)return()=>{be(o),X(n),r=!0};if(c==="then"){if(e.length===0)return{then:()=>o};const u=x(n,{type:"GET",path:e.map(f=>f.toString())}).then(S);return u.then.bind(u)}return V(n,[...e,c])},set(b,c,u){C(r);const[f,s]=U(u);return x(n,{type:"SET",path:[...e,c].map(g=>g.toString()),value:f},s).then(S)},apply(b,c,u){C(r);const f=e[e.length-1];if(f===k)return x(n,{type:"ENDPOINT"}).then(S);if(f==="bind")return V(n,e.slice(0,-1));const[s,g]=Q(u);return x(n,{type:"APPLY",path:e.map(y=>y.toString()),argumentList:s},g).then(S)},construct(b,c){C(r);const[u,f]=Q(c);return x(n,{type:"CONSTRUCT",path:e.map(s=>s.toString()),argumentList:u},f).then(S)}});return fe(o,n),o}function de(n){return Array.prototype.concat.apply([],n)}function Q(n){const e=n.map(U);return[e.map(t=>t[0]),de(e.map(t=>t[1]))]}const q=new WeakMap;function ge(n,e){return q.set(n,e),n}function we(n){return Object.assign(n,{[R]:!0})}function U(n){for(const[e,t]of Y)if(t.canHandle(n)){const[r,o]=t.serialize(n);return[{type:"HANDLER",name:e,value:r},o]}return[{type:"RAW",value:n},q.get(n)||[]]}function S(n){switch(n.type){case"HANDLER":return Y.get(n.name).deserialize(n.value);case"RAW":return n.value}}function x(n,e,t){return new Promise(r=>{const o=le();n.addEventListener("message",function b(c){!c.data||!c.data.id||c.data.id!==o||(n.removeEventListener("message",b),r(c.data))}),n.start&&n.start(),n.postMessage(Object.assign({id:o},e),t)})}function le(){return new Array(4).fill(0).map(()=>Math.floor(Math.random()*Number.MAX_SAFE_INTEGER).toString(16)).join("-")}let i;const E=new Array(128).fill(void 0);E.push(void 0,null,!0,!1);function _(n){return E[n]}let M=E.length;function ye(n){n<132||(E[n]=M,M=n)}function h(n){const e=_(n);return ye(n),e}let v=0,T=null;function F(){return(T===null||T.byteLength===0)&&(T=new Uint8Array(i.memory.buffer)),T}const D=typeof TextEncoder<"u"?new TextEncoder("utf-8"):{encode:()=>{throw Error("TextEncoder not available")}},me=typeof D.encodeInto=="function"?function(n,e){return D.encodeInto(n,e)}:function(n,e){const t=D.encode(n);return e.set(t),{read:n.length,written:t.length}};function O(n,e,t){if(t===void 0){const u=D.encode(n),f=e(u.length,1)>>>0;return F().subarray(f,f+u.length).set(u),v=u.length,f}let r=n.length,o=e(r,1)>>>0;const b=F();let c=0;for(;c<r;c++){const u=n.charCodeAt(c);if(u>127)break;b[o+c]=u}if(c!==r){c!==0&&(n=n.slice(c)),o=t(o,r,r=c+n.length*3,1)>>>0;const u=F().subarray(o+c,o+r),f=me(n,u);c+=f.written,o=t(o,r,c,1)>>>0}return v=c,o}function p(n){return n==null}let B=null;function d(){return(B===null||B.byteLength===0)&&(B=new Int32Array(i.memory.buffer)),B}const ee=typeof TextDecoder<"u"?new TextDecoder("utf-8",{ignoreBOM:!0,fatal:!0}):{decode:()=>{throw Error("TextDecoder not available")}};typeof TextDecoder<"u"&&ee.decode();function l(n,e){return n=n>>>0,ee.decode(F().subarray(n,n+e))}function a(n){M===E.length&&E.push(E.length+1);const e=M;return M=E[e],E[e]=n,e}let L=null;function ne(){return(L===null||L.byteLength===0)&&(L=new Float64Array(i.memory.buffer)),L}let N=null;function he(){return(N===null||N.byteLength===0)&&(N=new BigInt64Array(i.memory.buffer)),N}function $(n){const e=typeof n;if(e=="number"||e=="boolean"||n==null)return`${n}`;if(e=="string")return`"${n}"`;if(e=="symbol"){const o=n.description;return o==null?"Symbol":`Symbol(${o})`}if(e=="function"){const o=n.name;return typeof o=="string"&&o.length>0?`Function(${o})`:"Function"}if(Array.isArray(n)){const o=n.length;let b="[";o>0&&(b+=$(n[0]));for(let c=1;c<o;c++)b+=", "+$(n[c]);return b+="]",b}const t=/\[object ([^\]]+)\]/.exec(toString.call(n));let r;if(t.length>1)r=t[1];else return toString.call(n);if(r=="Object")try{return"Object("+JSON.stringify(n)+")"}catch{return"Object"}return n instanceof Error?`${n.name}: ${n.message}
${n.stack}`:r}const te=typeof FinalizationRegistry>"u"?{register:()=>{},unregister:()=>{}}:new FinalizationRegistry(n=>{i.__wbindgen_export_2.get(n.dtor)(n.a,n.b)});function re(n,e,t,r){const o={a:n,b:e,cnt:1,dtor:t},b=(...c)=>{o.cnt++;const u=o.a;o.a=0;try{return r(u,o.b,...c)}finally{--o.cnt===0?(i.__wbindgen_export_2.get(o.dtor)(u,o.b),te.unregister(o)):o.a=u}};return b.original=o,te.register(b,o,o),b}function pe(n,e,t){i._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__ha0b106c86a4071f7(n,e,a(t))}function ve(n,e,t){i._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h4a3a4bb5c4c0b039(n,e,a(t))}function ke(n,e){const t=e(n.length*1,1)>>>0;return F().set(n,t/1),v=n.length,t}function Ee(n,e){return n=n>>>0,ne().subarray(n/8,n/8+e)}function w(n,e){try{return n.apply(this,e)}catch(t){i.__wbindgen_exn_store(a(t))}}function Re(n,e,t,r){i.wasm_bindgen__convert__closures__invoke2_mut__h5b77cba7b0f68717(n,e,a(t),a(r))}typeof FinalizationRegistry>"u"||new FinalizationRegistry(n=>i.__wbg_intounderlyingbytesource_free(n>>>0)),typeof FinalizationRegistry>"u"||new FinalizationRegistry(n=>i.__wbg_intounderlyingsink_free(n>>>0)),typeof FinalizationRegistry>"u"||new FinalizationRegistry(n=>i.__wbg_intounderlyingsource_free(n>>>0));const oe=typeof FinalizationRegistry>"u"?{register:()=>{},unregister:()=>{}}:new FinalizationRegistry(n=>i.__wbg_mapmodel_free(n>>>0));class j{static __wrap(e){e=e>>>0;const t=Object.create(j.prototype);return t.__wbg_ptr=e,oe.register(t,t.__wbg_ptr,t),t}__destroy_into_raw(){const e=this.__wbg_ptr;return this.__wbg_ptr=0,oe.unregister(this),e}free(){const e=this.__destroy_into_raw();i.__wbg_mapmodel_free(e)}constructor(e,t,r,o,b){const c=ke(e,i.__wbindgen_malloc),u=v;var f=p(r)?0:O(r,i.__wbindgen_malloc,i.__wbindgen_realloc),s=v,g=p(o)?0:O(o,i.__wbindgen_malloc,i.__wbindgen_realloc),y=v;const m=i.mapmodel_new(c,u,t,f,s,g,y,p(b)?0:a(b));return h(m)}renderDebug(){let e,t;try{const s=i.__wbindgen_add_to_stack_pointer(-16);i.mapmodel_renderDebug(s,this.__wbg_ptr);var r=d()[s/4+0],o=d()[s/4+1],b=d()[s/4+2],c=d()[s/4+3],u=r,f=o;if(c)throw u=0,f=0,h(b);return e=u,t=f,l(u,f)}finally{i.__wbindgen_add_to_stack_pointer(16),i.__wbindgen_free(e,t,1)}}renderAmenities(){let e,t;try{const s=i.__wbindgen_add_to_stack_pointer(-16);i.mapmodel_renderAmenities(s,this.__wbg_ptr);var r=d()[s/4+0],o=d()[s/4+1],b=d()[s/4+2],c=d()[s/4+3],u=r,f=o;if(c)throw u=0,f=0,h(b);return e=u,t=f,l(u,f)}finally{i.__wbindgen_add_to_stack_pointer(16),i.__wbindgen_free(e,t,1)}}getInvertedBoundary(){let e,t;try{const s=i.__wbindgen_add_to_stack_pointer(-16);i.mapmodel_getInvertedBoundary(s,this.__wbg_ptr);var r=d()[s/4+0],o=d()[s/4+1],b=d()[s/4+2],c=d()[s/4+3],u=r,f=o;if(c)throw u=0,f=0,h(b);return e=u,t=f,l(u,f)}finally{i.__wbindgen_add_to_stack_pointer(16),i.__wbindgen_free(e,t,1)}}getBounds(){try{const o=i.__wbindgen_add_to_stack_pointer(-16);i.mapmodel_getBounds(o,this.__wbg_ptr);var e=d()[o/4+0],t=d()[o/4+1],r=Ee(e,t).slice();return i.__wbindgen_free(e,t*8,8),r}finally{i.__wbindgen_add_to_stack_pointer(16)}}renderZones(){let e,t;try{const s=i.__wbindgen_add_to_stack_pointer(-16);i.mapmodel_renderZones(s,this.__wbg_ptr);var r=d()[s/4+0],o=d()[s/4+1],b=d()[s/4+2],c=d()[s/4+3],u=r,f=o;if(c)throw u=0,f=0,h(b);return e=u,t=f,l(u,f)}finally{i.__wbindgen_add_to_stack_pointer(16),i.__wbindgen_free(e,t,1)}}isochrone(e){let t,r;try{const g=i.__wbindgen_add_to_stack_pointer(-16);i.mapmodel_isochrone(g,this.__wbg_ptr,a(e));var o=d()[g/4+0],b=d()[g/4+1],c=d()[g/4+2],u=d()[g/4+3],f=o,s=b;if(u)throw f=0,s=0,h(c);return t=f,r=s,l(f,s)}finally{i.__wbindgen_add_to_stack_pointer(16),i.__wbindgen_free(t,r,1)}}route(e){let t,r;try{const g=i.__wbindgen_add_to_stack_pointer(-16);i.mapmodel_route(g,this.__wbg_ptr,a(e));var o=d()[g/4+0],b=d()[g/4+1],c=d()[g/4+2],u=d()[g/4+3],f=o,s=b;if(u)throw f=0,s=0,h(c);return t=f,r=s,l(f,s)}finally{i.__wbindgen_add_to_stack_pointer(16),i.__wbindgen_free(t,r,1)}}bufferRoute(e){let t,r;try{const g=i.__wbindgen_add_to_stack_pointer(-16);i.mapmodel_bufferRoute(g,this.__wbg_ptr,a(e));var o=d()[g/4+0],b=d()[g/4+1],c=d()[g/4+2],u=d()[g/4+3],f=o,s=b;if(u)throw f=0,s=0,h(c);return t=f,r=s,l(f,s)}finally{i.__wbindgen_add_to_stack_pointer(16),i.__wbindgen_free(t,r,1)}}score(e,t){let r,o;try{const y=i.__wbindgen_add_to_stack_pointer(-16);i.mapmodel_score(y,this.__wbg_ptr,a(e),p(t)?0:a(t));var b=d()[y/4+0],c=d()[y/4+1],u=d()[y/4+2],f=d()[y/4+3],s=b,g=c;if(f)throw s=0,g=0,h(u);return r=s,o=g,l(s,g)}finally{i.__wbindgen_add_to_stack_pointer(16),i.__wbindgen_free(r,o,1)}}snapRoute(e){let t,r;try{const g=i.__wbindgen_add_to_stack_pointer(-16);i.mapmodel_snapRoute(g,this.__wbg_ptr,a(e));var o=d()[g/4+0],b=d()[g/4+1],c=d()[g/4+2],u=d()[g/4+3],f=o,s=b;if(u)throw f=0,s=0,h(c);return t=f,r=s,l(f,s)}finally{i.__wbindgen_add_to_stack_pointer(16),i.__wbindgen_free(t,r,1)}}}async function Ae(n,e){if(typeof Response=="function"&&n instanceof Response){if(typeof WebAssembly.instantiateStreaming=="function")try{return await WebAssembly.instantiateStreaming(n,e)}catch(r){if(n.headers.get("Content-Type")!="application/wasm")console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n",r);else throw r}const t=await n.arrayBuffer();return await WebAssembly.instantiate(t,e)}else{const t=await WebAssembly.instantiate(n,e);return t instanceof WebAssembly.Instance?{instance:t,module:n}:t}}function Se(){const n={};return n.wbg={},n.wbg.__wbindgen_object_drop_ref=function(e){h(e)},n.wbg.__wbindgen_string_get=function(e,t){const r=_(t),o=typeof r=="string"?r:void 0;var b=p(o)?0:O(o,i.__wbindgen_malloc,i.__wbindgen_realloc),c=v;d()[e/4+1]=c,d()[e/4+0]=b},n.wbg.__wbindgen_boolean_get=function(e){const t=_(e);return typeof t=="boolean"?t?1:0:2},n.wbg.__wbg_mapmodel_new=function(e){const t=j.__wrap(e);return a(t)},n.wbg.__wbindgen_string_new=function(e,t){const r=l(e,t);return a(r)},n.wbg.__wbindgen_is_bigint=function(e){return typeof _(e)=="bigint"},n.wbg.__wbindgen_bigint_from_u64=function(e){const t=BigInt.asUintN(64,e);return a(t)},n.wbg.__wbindgen_jsval_eq=function(e,t){return _(e)===_(t)},n.wbg.__wbindgen_error_new=function(e,t){const r=new Error(l(e,t));return a(r)},n.wbg.__wbindgen_is_object=function(e){const t=_(e);return typeof t=="object"&&t!==null},n.wbg.__wbindgen_is_undefined=function(e){return _(e)===void 0},n.wbg.__wbindgen_in=function(e,t){return _(e)in _(t)},n.wbg.__wbindgen_number_get=function(e,t){const r=_(t),o=typeof r=="number"?r:void 0;ne()[e/8+1]=p(o)?0:o,d()[e/4+0]=!p(o)},n.wbg.__wbindgen_jsval_loose_eq=function(e,t){return _(e)==_(t)},n.wbg.__wbindgen_object_clone_ref=function(e){const t=_(e);return a(t)},n.wbg.__wbg_getwithrefkey_3b3c46ba20582127=function(e,t){const r=_(e)[_(t)];return a(r)},n.wbg.__wbg_new_abda76e883ba8a5f=function(){const e=new Error;return a(e)},n.wbg.__wbg_stack_658279fe44541cf6=function(e,t){const r=_(t).stack,o=O(r,i.__wbindgen_malloc,i.__wbindgen_realloc),b=v;d()[e/4+1]=b,d()[e/4+0]=o},n.wbg.__wbg_error_f851667af71bcfc6=function(e,t){let r,o;try{r=e,o=t,console.error(l(e,t))}finally{i.__wbindgen_free(r,o,1)}},n.wbg.__wbg_performance_a1b8bde2ee512264=function(e){const t=_(e).performance;return a(t)},n.wbg.__wbg_now_abd80e969af37148=function(e){return _(e).now()},n.wbg.__wbindgen_cb_drop=function(e){const t=h(e).original;return t.cnt--==1?(t.a=0,!0):!1},n.wbg.__wbg_fetch_bc7c8e27076a5c84=function(e){const t=fetch(_(e));return a(t)},n.wbg.__wbg_done_2ffa852272310e47=function(e){return _(e).done},n.wbg.__wbg_getReader_ab94afcb5cb7689a=function(){return w(function(e){const t=_(e).getReader();return a(t)},arguments)},n.wbg.__wbg_value_9f6eeb1e2aab8d96=function(e){const t=_(e).value;return a(t)},n.wbg.__wbg_fetch_1e4e8ed1f64c7e28=function(e){const t=fetch(_(e));return a(t)},n.wbg.__wbg_queueMicrotask_3cbae2ec6b6cd3d6=function(e){const t=_(e).queueMicrotask;return a(t)},n.wbg.__wbindgen_is_function=function(e){return typeof _(e)=="function"},n.wbg.__wbg_queueMicrotask_481971b0d87f3dd4=function(e){queueMicrotask(_(e))},n.wbg.__wbg_fetch_693453ca3f88c055=function(e,t){const r=_(e).fetch(_(t));return a(r)},n.wbg.__wbg_debug_34c9290896ec9856=function(e){console.debug(_(e))},n.wbg.__wbg_error_e60eff06f24ab7a4=function(e){console.error(_(e))},n.wbg.__wbg_info_d7d58472d0bab115=function(e){console.info(_(e))},n.wbg.__wbg_log_a4530b4fe289336f=function(e){console.log(_(e))},n.wbg.__wbg_warn_f260f49434e45e62=function(e){console.warn(_(e))},n.wbg.__wbg_newwithstrandinit_f581dff0d19a8b03=function(){return w(function(e,t,r){const o=new Request(l(e,t),_(r));return a(o)},arguments)},n.wbg.__wbg_instanceof_Response_4c3b1446206114d1=function(e){let t;try{t=_(e)instanceof Response}catch{t=!1}return t},n.wbg.__wbg_url_83a6a4f65f7a2b38=function(e,t){const r=_(t).url,o=O(r,i.__wbindgen_malloc,i.__wbindgen_realloc),b=v;d()[e/4+1]=b,d()[e/4+0]=o},n.wbg.__wbg_status_d6d47ad2837621eb=function(e){return _(e).status},n.wbg.__wbg_headers_24def508a7518df9=function(e){const t=_(e).headers;return a(t)},n.wbg.__wbg_body_69be35dff3d68d53=function(e){const t=_(e).body;return p(t)?0:a(t)},n.wbg.__wbg_arrayBuffer_5b2688e3dd873fed=function(){return w(function(e){const t=_(e).arrayBuffer();return a(t)},arguments)},n.wbg.__wbg_signal_3c701f5f40a5f08d=function(e){const t=_(e).signal;return a(t)},n.wbg.__wbg_new_0ae46f44b7485bb2=function(){return w(function(){const e=new AbortController;return a(e)},arguments)},n.wbg.__wbg_abort_2c4fb490d878d2b2=function(e){_(e).abort()},n.wbg.__wbg_byobRequest_05466bb0cacd89fa=function(e){const t=_(e).byobRequest;return p(t)?0:a(t)},n.wbg.__wbg_close_d29a75e8efc5fa94=function(){return w(function(e){_(e).close()},arguments)},n.wbg.__wbg_view_1fe68975176283b3=function(e){const t=_(e).view;return p(t)?0:a(t)},n.wbg.__wbg_respond_6272b341f88864a2=function(){return w(function(e,t){_(e).respond(t>>>0)},arguments)},n.wbg.__wbg_read_79c1f6a58844174c=function(e){const t=_(e).read();return a(t)},n.wbg.__wbg_releaseLock_6eb6fa75435874b8=function(e){_(e).releaseLock()},n.wbg.__wbg_cancel_ef8b2c6f99da9cde=function(e){const t=_(e).cancel();return a(t)},n.wbg.__wbg_new_7a20246daa6eec7e=function(){return w(function(){const e=new Headers;return a(e)},arguments)},n.wbg.__wbg_append_aa3f462f9e2b5ff2=function(){return w(function(e,t,r,o,b){_(e).append(l(t,r),l(o,b))},arguments)},n.wbg.__wbg_close_79df9bcee94a607c=function(){return w(function(e){_(e).close()},arguments)},n.wbg.__wbg_enqueue_e8019641f9877e27=function(){return w(function(e,t){_(e).enqueue(_(t))},arguments)},n.wbg.__wbg_get_bd8e338fbd5f5cc8=function(e,t){const r=_(e)[t>>>0];return a(r)},n.wbg.__wbg_length_cd7af8117672b8b8=function(e){return _(e).length},n.wbg.__wbg_newnoargs_e258087cd0daa0ea=function(e,t){const r=new Function(l(e,t));return a(r)},n.wbg.__wbg_next_40fc327bfc8770e6=function(e){const t=_(e).next;return a(t)},n.wbg.__wbg_next_196c84450b364254=function(){return w(function(e){const t=_(e).next();return a(t)},arguments)},n.wbg.__wbg_done_298b57d23c0fc80c=function(e){return _(e).done},n.wbg.__wbg_value_d93c65011f51a456=function(e){const t=_(e).value;return a(t)},n.wbg.__wbg_iterator_2cee6dadfd956dfa=function(){return a(Symbol.iterator)},n.wbg.__wbg_get_e3c254076557e348=function(){return w(function(e,t){const r=Reflect.get(_(e),_(t));return a(r)},arguments)},n.wbg.__wbg_call_27c0f87801dedf93=function(){return w(function(e,t){const r=_(e).call(_(t));return a(r)},arguments)},n.wbg.__wbg_new_72fb9a18b5ae2624=function(){const e=new Object;return a(e)},n.wbg.__wbg_self_ce0dbfc45cf2f5be=function(){return w(function(){const e=self.self;return a(e)},arguments)},n.wbg.__wbg_window_c6fb939a7f436783=function(){return w(function(){const e=window.window;return a(e)},arguments)},n.wbg.__wbg_globalThis_d1e6af4856ba331b=function(){return w(function(){const e=globalThis.globalThis;return a(e)},arguments)},n.wbg.__wbg_global_207b558942527489=function(){return w(function(){const e=global.global;return a(e)},arguments)},n.wbg.__wbg_isArray_2ab64d95e09ea0ae=function(e){return Array.isArray(_(e))},n.wbg.__wbg_instanceof_ArrayBuffer_836825be07d4c9d2=function(e){let t;try{t=_(e)instanceof ArrayBuffer}catch{t=!1}return t},n.wbg.__wbg_new_28c511d9baebfa89=function(e,t){const r=new Error(l(e,t));return a(r)},n.wbg.__wbg_call_b3ca7c6051f9bec1=function(){return w(function(e,t,r){const o=_(e).call(_(t),_(r));return a(o)},arguments)},n.wbg.__wbg_isSafeInteger_f7b04ef02296c4d2=function(e){return Number.isSafeInteger(_(e))},n.wbg.__wbg_new_81740750da40724f=function(e,t){try{var r={a:e,b:t},o=(c,u)=>{const f=r.a;r.a=0;try{return Re(f,r.b,c,u)}finally{r.a=f}};const b=new Promise(o);return a(b)}finally{r.a=r.b=0}},n.wbg.__wbg_resolve_b0083a7967828ec8=function(e){const t=Promise.resolve(_(e));return a(t)},n.wbg.__wbg_catch_0260e338d10f79ae=function(e,t){const r=_(e).catch(_(t));return a(r)},n.wbg.__wbg_then_0c86a60e8fcfe9f6=function(e,t){const r=_(e).then(_(t));return a(r)},n.wbg.__wbg_then_a73caa9a87991566=function(e,t,r){const o=_(e).then(_(t),_(r));return a(o)},n.wbg.__wbg_buffer_12d079cc21e14bdb=function(e){const t=_(e).buffer;return a(t)},n.wbg.__wbg_newwithbyteoffsetandlength_aa4a17c33a06e5cb=function(e,t,r){const o=new Uint8Array(_(e),t>>>0,r>>>0);return a(o)},n.wbg.__wbg_new_63b92bc8671ed464=function(e){const t=new Uint8Array(_(e));return a(t)},n.wbg.__wbg_set_a47bac70306a19a7=function(e,t,r){_(e).set(_(t),r>>>0)},n.wbg.__wbg_length_c20a40f15020d68a=function(e){return _(e).length},n.wbg.__wbg_instanceof_Uint8Array_2b3bbecd033d19f6=function(e){let t;try{t=_(e)instanceof Uint8Array}catch{t=!1}return t},n.wbg.__wbg_buffer_dd7f74bc60f1faab=function(e){const t=_(e).buffer;return a(t)},n.wbg.__wbg_byteLength_58f7b4fab1919d44=function(e){return _(e).byteLength},n.wbg.__wbg_byteOffset_81d60f7392524f62=function(e){return _(e).byteOffset},n.wbg.__wbg_has_0af94d20077affa2=function(){return w(function(e,t){return Reflect.has(_(e),_(t))},arguments)},n.wbg.__wbg_set_1f9b04f170055d33=function(){return w(function(e,t,r){return Reflect.set(_(e),_(t),_(r))},arguments)},n.wbg.__wbg_stringify_8887fe74e1c50d81=function(){return w(function(e){const t=JSON.stringify(_(e));return a(t)},arguments)},n.wbg.__wbindgen_bigint_get_as_i64=function(e,t){const r=_(t),o=typeof r=="bigint"?r:void 0;he()[e/8+1]=p(o)?BigInt(0):o,d()[e/4+0]=!p(o)},n.wbg.__wbindgen_debug_string=function(e,t){const r=$(_(t)),o=O(r,i.__wbindgen_malloc,i.__wbindgen_realloc),b=v;d()[e/4+1]=b,d()[e/4+0]=o},n.wbg.__wbindgen_throw=function(e,t){throw new Error(l(e,t))},n.wbg.__wbindgen_memory=function(){const e=i.memory;return a(e)},n.wbg.__wbindgen_closure_wrapper1915=function(e,t,r){const o=re(e,t,676,pe);return a(o)},n.wbg.__wbindgen_closure_wrapper2902=function(e,t,r){const o=re(e,t,815,ve);return a(o)},n}function xe(n,e){return i=n.exports,G.__wbindgen_wasm_module=e,N=null,L=null,B=null,T=null,i}async function G(n){if(i!==void 0)return i;typeof n>"u"&&(n="/15m/assets/backend_bg.wasm");const e=Se();(typeof n=="string"||typeof Request=="function"&&n instanceof Request||typeof URL=="function"&&n instanceof URL)&&(n=fetch(n));const{instance:t,module:r}=await Ae(await n,e);return xe(t,r)}class Oe{constructor(){_e(this,"inner");this.inner=null}async loadOsmFile(e,t,r,o){await G(),this.inner=await new j(e,!0,t,r,o)}async loadGraphFile(e){await G(),this.inner=await new j(e,!1,void 0,void 0,void 0)}isLoaded(){return this.inner!=null}unset(){this.inner=null}getBounds(){if(!this.inner)throw new Error("Backend used without a file loaded");return Array.from(this.inner.getBounds())}getInvertedBoundary(){if(!this.inner)throw new Error("Backend used without a file loaded");return JSON.parse(this.inner.getInvertedBoundary())}renderDebug(){if(!this.inner)throw new Error("Backend used without a file loaded");return JSON.parse(this.inner.renderDebug())}renderAmenities(){if(!this.inner)throw new Error("Backend used without a file loaded");return JSON.parse(this.inner.renderAmenities())}renderZones(){if(!this.inner)throw new Error("Backend used without a file loaded");return JSON.parse(this.inner.renderZones())}isochrone(e){if(!this.inner)throw new Error("Backend used without a file loaded");return JSON.parse(this.inner.isochrone({x:e.start.lng,y:e.start.lat,mode:e.mode,style:e.style,start_time:e.startTime,max_seconds:e.maxSeconds}))}route(e){if(!this.inner)throw new Error("Backend used without a file loaded");return JSON.parse(this.inner.route({x1:e.start.lng,y1:e.start.lat,x2:e.end[0],y2:e.end[1],mode:e.mode,debug_search:e.debugSearch,use_heuristic:e.useHeuristic,start_time:e.startTime}))}bufferRoute(e){if(!this.inner)throw new Error("Backend used without a file loaded");return JSON.parse(this.inner.bufferRoute({x1:e.start.lng,y1:e.start.lat,x2:e.end[0],y2:e.end[1],mode:e.mode,use_heuristic:e.useHeuristic,start_time:e.startTime,max_seconds:e.maxSeconds}))}score(e,t){if(!this.inner)throw new Error("Backend used without a file loaded");return JSON.parse(this.inner.score({poi_kinds:e.poiKinds,max_seconds:e.maxSeconds},t))}snapRoute(e){if(!this.inner)throw new Error("Backend used without a file loaded");return JSON.parse(this.inner.snapRoute({input:JSON.stringify(e.input),mode:e.mode}))}}J(Oe)})();
