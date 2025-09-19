import { lumo as compile } from "../wasm/web/lumo_wasm.js";
import { LumoWebLib } from "./stdlib.mjs";
import { read } from "./ffi.mjs";

export async function lumo(code) {
    const result = compile(code);
    const returnType = eval(result.return_type());
    const bytecodes = result.bytecode().buffer;

    const stdlib = new LumoWebLib();
    const { instance } = await WebAssembly.instantiate(bytecodes, {
        env: stdlib.bridge(),
    });
    stdlib.set_wasm(instance);

    const raw = instance.exports._start();
    if (returnType !== "void") {
        return read(instance, returnType, raw);
    }
}
