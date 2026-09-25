import "./styles.css";

import {transpile} from '../pkg';

const code_input = document.getElementById("code")! as HTMLTextAreaElement;
const output = document.getElementById("output")!;
const transpile_button = document.getElementById("button")! as HTMLInputElement;
const slider = document.getElementById("slider") as HTMLDivElement
const code_half = document.getElementById("code-half-container") as HTMLDivElement;

function clicked() {
    output.textContent = transpile(code_input.value);
}

transpile_button.onclick = clicked;

// https://stackoverflow.com/a/55565128
slider.onmousedown = function dragMouseDown(e) {
    e.preventDefault()
    let initial = e.clientX;
    let initial_width = code_half.getBoundingClientRect().width;
    document.onmousemove = function onMouseMove(e) {
        code_half.style.width = `${initial_width + e.clientX - initial}px`;
    }
    document.onmouseup = () => document.onmousemove = document.onmouseup = null;
}