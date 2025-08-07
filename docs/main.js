import { lumo } from "./runtime/web.mjs";
let timer;

const codeEditor = document.getElementById("code");
const resultArea = document.getElementById("result");
const runBtn = document.getElementById("run");
const buildBtn = document.getElementById("build");

const timerLabel = document.getElementById("timer");
const exampleFieldInput = document.getElementById("example-field");
const exampleSelectBtn = document.getElementById("example-select");

exampleSelectBtn.addEventListener("click", async () => {
    const exampleCodeUrl = `https://raw.githubusercontent.com/archy-none/lumo/refs/heads/master/example/${exampleFieldInput.value}.lm`;
    const response = await fetch(exampleCodeUrl);
    codeEditor.value = await response.text();
});

runBtn.addEventListener("click", async () => {
    resultArea.innerHTML = "";
    timerLabel.textContent = "";
    const startTime = Date.now();
    timer = setInterval(() => {
        timerLabel.textContent = `Time: ${Date.now() - startTime}ms`;
    }, 1);
    try {
        const result = await lumo(codeEditor.value);
        if (result !== undefined) {
            resultArea.innerHTML = JSON.stringify(result, null, 2);
        }
    } catch (error) {
        resultArea.innerHTML = error;
    }
    clearInterval(timer);
});

buildBtn.addEventListener("click", () => {
    const data = `
        <!DOCTYPE html>
        <html>
            <head>
                <title>Lumo App</title>
            </head>
            <body>
                <script type="module" src="https://archy-none.github.io/lumo/runtime/web.mjs"></\script>
                <lumo-code>${codeEditor.value}</lumo-code>
            </body>
        </html>
    `;
    const blob = new Blob([data], { type: "text/plain" });
    const link = document.createElement("a");
    link.href = URL.createObjectURL(blob);
    link.download = "lumo-app.html";
    link.click();
});
