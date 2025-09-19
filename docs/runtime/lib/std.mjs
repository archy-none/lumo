import { write, read, concatBytes } from "../ffi.mjs";

export class LumoStdLib {
    constructor() {
        let reads = (typ, val) => read(this.instance, typ, val);
        let writes = (typ, val) => write(this.instance, typ, val);
        this.functions = {
            to_str: (value) => writes("str", value.toString()),
            to_num: (value) => parseFloat(reads("str", value)),
            repeat: (value, count) => {
                value = reads("str", value).repeat(Math.floor(count));
                return writes("str", value);
            },
            concat: (str1, str2) => {
                str1 = reads("str", str1);
                str2 = reads("str", str2);
                return writes("str", str1 + str2);
            },
            strcmp: (str1, str2) => {
                str1 = reads("str", str1);
                str2 = reads("str", str2);
                return str1 === str2;
            },
            strlen: (str) => {
                str = reads("str", str);
                return str.length;
            },
            split: (str, delimiter) => {
                let value = reads("str", str).split(reads("str", delimiter));
                writes({ type: "array", element: "str" }, value);
            },
            array: (init, len) => {
                let value = Array(len).fill(init);
                writes({ type: "array", element: "int" }, value);
            },
            slice: (ptr, start, end) => {
                const array = reads({ type: "array", element: "int" }, ptr);
                const index = (i) => (i < 0 ? array.length + i : i);
                const value = array.slice(index(start), index(end));
                return writes({ type: "array", element: "int" }, value);
            },
            arrlen: (addr) => {
                let view = new Uint8Array(this.instance.exports.mem.buffer);
                return concatBytes(view.slice(addr, addr + 4), false);
            },
            join: (array, delimiter) => {
                array = reads({ type: "array", element: "str" }, array);
                writes("str", array.join(reads("str", delimiter)));
            },
            append: (a, b) => {
                let typ = { type: "array", element: "int" };
                return writes(typ, [...reads(typ, a), ...reads(typ, b)]);
            },
        };
    }
    set_wasm(instance) {
        this.instance = instance;
    }
    bridge() {
        const b = {};
        for (const k of Object.keys(this.functions)) {
            b[k] = (...a) => this.functions[k](...a);
        }
        return b;
    }
}

export class LumoNodeLib extends LumoStdLib {
    constructor() {
        super();
        this.functions.print = (message) => {
            console.log(read(this.instance, "str", message));
        };
        this.functions.write = (message) => {
            process.stdout.write(read(this.instance, "str", message));
        };
    }
}

export class LumoWebLib extends LumoStdLib {
    constructor() {
        super();
        this.functions.alert = (message) => {
            window.alert(read(this.instance, "str", message));
        };
        this.functions.confirm = (message) => {
            return window.confirm(read(this.instance, "str", message));
        };
        this.functions.prompt = (message) => {
            const answer = window.prompt(read(this.instance, "str", message));
            return write(this.instance, "str", answer);
        };
    }
}
