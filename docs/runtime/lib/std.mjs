import { write, read, concatBytes } from "../ffi.mjs";

export class LumoStdLib {
    constructor() {
        this.functions = {
            to_str: (value) => {
                return write(this.instance, "str", value.toString());
            },
            to_num: (value) => {
                return parseFloat(read(this.instance, "str", value));
            },
            repeat: (value, count) => {
                return write(
                    this.instance,
                    "str",
                    read(this.instance, "str", value).repeat(Math.floor(count)),
                );
            },
            concat: (str1, str2) => {
                str1 = read(this.instance, "str", str1);
                str2 = read(this.instance, "str", str2);
                return write(this.instance, "str", str1 + str2);
            },
            strcmp: (str1, str2) => {
                str1 = read(this.instance, "str", str1);
                str2 = read(this.instance, "str", str2);
                return str1 === str2;
            },
            strlen: (str) => {
                str = read(this.instance, "str", str);
                return str.length;
            },
            split: (str, delimiter) => {
                str = read(this.instance, "str", str);
                delimiter = read(this.instance, "str", delimiter);
                const splitted = str.split(delimiter);
                const typ = { type: "array", element: "str" };
                return write(this.instance, typ, splitted);
            },
            array: (init, len) => {
                return write(
                    this.instance,
                    { type: "array", element: "int" },
                    Array(len).fill(init),
                );
            },
            slice: (ptr, start, end) => {
                const typ = { type: "array", element: "int" };
                const array = read(this.instance, typ, ptr);
                const index = (i) => (i < 0 ? array.length + i : i);
                const slice = array.slice(index(start), index(end));
                return write(this.instance, typ, slice);
            },
            arrlen: (addr) => {
                const memoryView = new Uint8Array(
                    this.instance.exports.mem.buffer,
                );
                return concatBytes(memoryView.slice(addr, addr + 4), false);
            },
            join: (ptr, delimiter) => {
                const typ = { type: "array", element: "str" };
                const array = read(this.instance, typ, ptr);
                delimiter = read(this.instance, "str", delimiter);
                return write(this.instance, "str", array.join(delimiter));
            },
            append: (a, b) => {
                const typ = { type: "array", element: "int" };
                const array1 = read(this.instance, typ, a);
                const array2 = read(this.instance, typ, b);
                return write(this.instance, typ, [...array1, ...array2]);
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
