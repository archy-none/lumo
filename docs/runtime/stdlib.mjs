import { write, read, concatBytes } from "./ffi.mjs";

export class LumoStdLib {
    constructor() {
        this.reads = (typ, val) => read(this.instance, typ, val);
        this.writes = (typ, val) => write(this.instance, typ, val);
        this.functions = {
            to_str: (value) => this.writes("str", value.toString()),
            to_num: (value) => parseFloat(this.reads("str", value)),
            repeat: (value, count) => {
                value = this.reads("str", value).repeat(Math.floor(count));
                return this.writes("str", value);
            },
            concat: (str1, str2) => {
                let str = this.reads("str", str1) + this.reads("str", str2);
                return this.writes("str", str);
            },
            strcmp: (str1, str2) => {
                return this.reads("str", str1) === this.reads("str", str2);
            },
            strlen: (str) => this.reads("str", str).length,
            split: (str, delimiter) => {
                delimiter = this.reads("str", delimiter);
                let value = this.reads("str", str).split(delimiter);
                this.writes({ type: "array", element: "str" }, value);
            },
            array: (init, len) => {
                let value = Array(len).fill(init);
                this.writes({ type: "array", element: "int" }, value);
            },
            slice: (ptr, start, end) => {
                const typ = { type: "array", element: "int" };
                const index = (i) => (i < 0 ? array.length + i : i);
                const array = this.reads(typ, ptr);
                const result = array.slice(index(start), index(end));
                return this.writes(typ, result);
            },
            arrlen: (addr) => {
                let view = new Uint8Array(this.instance.exports.mem.buffer);
                return concatBytes(view.slice(addr, addr + 4), false);
            },
            join: (array, delimiter) => {
                array = this.reads({ type: "array", element: "str" }, array);
                this.writes("str", array.join(reads("str", delimiter)));
            },
            append: (a, b) => {
                let typ = { type: "array", element: "int" };
                return this.writes(typ, [
                    ...this.reads(typ, a),
                    ...this.reads(typ, b),
                ]);
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
            console.log(this.reads("str", message));
        };
        this.functions.write = (message) => {
            process.stdout.write(this.reads("str", message));
        };
    }
}

export class LumoWebLib extends LumoStdLib {
    constructor() {
        super();
        this.functions.alert = (message) => {
            window.alert(this.reads("str", message));
        };
        this.functions.confirm = (message) => {
            return window.confirm(this.reads("str", message));
        };
        this.functions.prompt = (message) => {
            const answer = window.prompt(this.reads("str", message));
            return this.writes("str", answer);
        };
    }
}
