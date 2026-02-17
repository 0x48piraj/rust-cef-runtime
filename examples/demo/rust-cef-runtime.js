const rawInvoke = window.core.invoke;

export async function invoke(cmd, payload = {}) {
const res = await rawInvoke(cmd, JSON.stringify(payload));
return JSON.parse(res);
}

export async function invokeVoid(cmd, payload = {}) {
await rawInvoke(cmd, JSON.stringify(payload));
}

export async function invokeText(cmd, text = "") {
return rawInvoke(cmd, text);
}
