const rawInvoke = window.core.invoke;

export async function invoke(command, payload = null) {
const response = await rawInvoke(command, JSON.stringify(payload));
return JSON.parse(response);
}
