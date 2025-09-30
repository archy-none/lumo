import { readFileSync } from "node:fs";
import { createInterface } from "readline";
import { lumo } from "./docs/runtime/node.mjs";

async function runScript(source) {
    try {
        const result = await lumo(source.toString());
        if (result !== undefined) console.log(result);
    } catch (e) {
        console.log("\u0007Error!", e);
    }
}

function repl(lib = "") {
    let code = lib;
    const rl = createInterface({
        input: process.stdin,
        output: process.stdout,
        prompt: "> ",
    });

    console.log("Lumo REPL");
    rl.prompt();

    rl.on("line", async (input) => {
        if (input.trim() !== "") {
            try {
                let result = await lumo(`${code};${input}`);
                if (result !== undefined) console.log(result);
                code += `;${input}`;
            } catch (e) {
                console.log("\u0007Error!", e);
            }
        }
        rl.prompt();
    });

    rl.on("close", () => {
        console.log("Bye");
        process.exit(0);
    });
}

const args = process.argv.slice(2);
if (args.length === 0) {
    repl();
} else if (args.length === 2 && (args[0] === "--line" || args[0] === "-l")) {
    runScript(args[1]);
} else if (args.length === 2 && (args[0] === "--debug" || args[0] === "-d")) {
    repl(readFileSync(args[1], "utf8"));
} else if (args.length === 1) {
    runScript(readFileSync(args[0], "utf8"));
}
