import { lumo as compile } from "../wasm/node/lumo_wasm.js";
import { LumoNodeLib } from "./lib/std.mjs";
import { LumoMathLib } from "./lib/math.mjs";
import { LumoOSLib } from "./lib/os.mjs";
import { LumoRandomLib } from "./lib/random.mjs";
import { LumoDatetimeLib } from "./lib/datetime.mjs";
import { LumoTimeLib } from "./lib/time.mjs";
import { module } from "./module.mjs";
import { read } from "./ffi.mjs";

const moduleClasses = {
    math: LumoMathLib,
    os: LumoOSLib,
    random: LumoRandomLib,
    datetime: LumoDatetimeLib,
    time: LumoTimeLib,
};

export async function lumo(code, customModules = {}) {
    const result = compile(code);
    const returnType = eval(`(${result.get_return_type()})`);
    const bytecodes = result.get_bytecode().buffer;
    const moduleObj = await WebAssembly.compile(bytecodes);
    const importsInfo = WebAssembly.Module.imports(moduleObj);
    const stdLib = new LumoNodeLib();
    const importObject = { env: { ...stdLib.bridge() } };
    const instances = { LumoNodeLib: stdLib };
    module({
        importsInfo,
        moduleClasses,
        customModules,
        instances,
        importObject,
        runtime: "Node",
    });

    const wab = bytecodes;
    const { instance } = await WebAssembly.instantiate(wab, importObject);
    Object.values(instances).forEach((inst) => inst.set_wasm(instance));
    const raw = instance.exports._start();
    if (returnType != null) {
        return read(instance, returnType, raw);
    }
}
