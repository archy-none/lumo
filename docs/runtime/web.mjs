import init, { lumo as compile } from "../wasm/web/lumo_wasm.js";
import { LumoWebLib } from "./lib/std.mjs";
import { LumoMathLib } from "./lib/math.mjs";
import { LumoRandomLib } from "./lib/random.mjs";
import { LumoDatetimeLib } from "./lib/datetime.mjs";
import { LumoTimeLib } from "./lib/time.mjs";
import { module } from "./module.mjs";
import { read } from "./ffi.mjs";

const moduleClasses = {
    math: LumoMathLib,
    random: LumoRandomLib,
    datetime: LumoDatetimeLib,
    time: LumoTimeLib,
};

await init();
export async function lumo(code, customModules = {}) {
    const result = compile(code);
    const returnType = eval(`(${result.get_return_type()})`);
    const bytecodes = result.get_bytecode().buffer;
    const moduleObj = await WebAssembly.compile(bytecodes);
    const importsInfo = WebAssembly.Module.imports(moduleObj);
    const stdLib = new LumoWebLib();
    const importObject = { env: { ...stdLib.bridge() } };
    const instances = { LumoWebLib: stdLib };
    module({
        importsInfo,
        moduleClasses,
        customModules,
        instances,
        importObject,
        runtime: "Web",
    });

    const wab = bytecodes;
    const { instance } = await WebAssembly.instantiate(wab, importObject);
    Object.values(instances).forEach((inst) => inst.set_wasm(instance));
    const raw = instance.exports._start();
    if (returnType != null) {
        return read(instance, returnType, raw);
    }
}

class Lumo extends HTMLElement {
    constructor() {
        super();
        console.log("Welcome to the Lumo programming!");
    }

    async connectedCallback() {
        await lumo(this.innerHTML);
        this.remove();
    }
}

customElements.define("lumo-code", Lumo);
